use crate::errors::{Errors, Result};
use parking_lot::{Condvar, Mutex, RwLock};
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    fmt::Debug,
    sync::Arc,
};
/// `CellMap` used to support concurrent channel
#[derive(Debug)]
pub(crate) struct CellMap<K, V>(RefCell<HashMap<K, V>>);
unsafe impl<K, V> Sync for CellMap<K, V> {}
unsafe impl<K, V> Send for CellMap<K, V> {}

use super::linked_list::{Cell, List, ListNode};
/// Key is a struct type, we use it as the
/// message's key
#[derive(Debug)]
pub(crate) struct Key<T>(String, Option<Arc<MspcChannel<T>>>);

impl<T> Drop for Key<T> {
    fn drop(&mut self) {
        // get write_guard
        if let Some(ref channel) = self.1 {
            let mut write_guard = channel.counter.write();
            // write_guard
            let _ = write_guard.remove(&self.0);
            // update key in duplicate
            let mut write_guard_in_duplicate = channel.key_message_id_in_duplicate.0.borrow_mut();
            // update ref_count
            let mut write_ref_count = channel.id_to_message.0.borrow_mut();
            // get list_guard
            let mut list_guard = channel.cached_messages.lock();
            // get id_to_node guard
            let id_to_node_guard = channel.id_to_node.0.borrow_mut();
            let set_opt = write_guard_in_duplicate.get(&self.0);
            if let Some(set) = set_opt {
                for id in set.iter() {
                    let message_opt = write_ref_count.get_mut(id);
                    if let Some(message_) = message_opt {
                        let mut ref_guard = message_.ref_count.write();
                        *ref_guard = ref_guard.wrapping_sub(1);
                        if *ref_guard == 0 {
                            let node_opt = id_to_node_guard.get(id);
                            if let Some(node) = node_opt {
                                list_guard.remove(&Arc::<Cell>::clone(node));
                                // no_duplicate_key will be first
                                list_guard.list_push_first(&Arc::<Cell>::clone(node));
                            }
                        }
                    }
                }
            }
            let _ = write_guard_in_duplicate.remove(&self.0);
        }
    }
}

/// `InternalMessage` is a struct which is used
/// to passed by channel
#[derive(Debug)]
pub struct InternalMessage<T> {
    /// a message with multi-keys
    keys: Vec<Key<T>>,
    /// a message data
    pub data: T,
    /// duplicate times
    ref_count: RwLock<i32>,
    /// message_id
    id: i32,
}

impl<T> InternalMessage<T> {
    /// `new` is used to generate a `InternalMessage`
    pub fn new(vecs: Vec<String>, data_: T) -> InternalMessage<T> {
        let mut message = InternalMessage {
            keys: Vec::<Key<T>>::new(),
            data: data_,
            ref_count: RwLock::new(0),
            id: 0,
        };
        for vec in vecs {
            message.keys.push(Key(vec, None));
        }
        message
    }
}
#[derive(Debug, Clone)]
/// `Sender` is used to recieve message from channel.
pub struct Sender<T> {
    /// hold a channel ref_counter
    chan: Arc<MspcChannel<T>>,
}

impl<T> Sender<T> {
    /// send a message to the channel
    pub fn send(&self, mut message: InternalMessage<T>) -> Result<()> {
        for key in &mut message.keys {
            key.1 = Some(Arc::clone(&self.chan));
        }
        let mut mutex = self.chan.message_id.lock();
        *mutex = mutex.wrapping_add(1);
        message.id = *mutex;
        drop(mutex);
        // -1 means this is an unbounded channel
        if self.chan.bounded_size == -1 {
            self.chan.push_message(message);
            Ok(())
        } else {
            loop {
                // get lock make sure operations are atomic
                let mut lock = self.chan.message_id.lock();
                let mut write_guard = self.chan.cached_messages.lock();
                if write_guard.list_count() < self.chan.bounded_size {
                    drop(write_guard);
                    self.chan.push_message(message);
                    write_guard = self.chan.cached_messages.lock();
                    if write_guard.list_count() == 1 {
                        let _ = self.chan.cond_var_recieve.notify_one();
                    }
                    return Ok(());
                }
                // channel is full, wait here.
                self.chan.cond_var_send.wait(&mut lock);
            }
        }
    }
}

#[derive(Debug)]
#[deny(clippy::clone_on_ref_ptr)]
/// `Reciever` is used to recieve message from channel.
pub struct Reciever<T> {
    /// hold a channel ref_counter
    chan: Arc<MspcChannel<T>>,
}

impl<T> Reciever<T>
where
    T: Debug,
{
    /// `recv` recieve message from channel
    pub fn recv(&self) -> Result<InternalMessage<T>> {
        // get write_guard
        loop {
            let mut write_guard = self.chan.cached_messages.lock();
            // 1.there is no message in channel
            // just loop ahead
            if write_guard.list_count() == 0 {
                self.chan.cond_var_recieve.wait(&mut write_guard);
                continue;
            }

            // 2.check is there a valid message
            let message_id = write_guard.list_first();
            let size = write_guard.list_count();
            drop(write_guard);
            // if valid, we should give it out
            if self.chan.is_valid(message_id) {
                // do some necessary update
                let message = self.chan.remove(message_id);
                if 1 == self.chan.bounded_size.wrapping_sub(size) {
                    let _ = self.chan.cond_var_send.notify_all();
                }
                return Ok(message);
            }
            // otherwise, all messages are duplicated,
            // return error
            return Err(Errors::KeyDuplicate);
        }
    }
}

/// `MspcChannel` is a multi producer and single consumer
/// channel, we will use it to transfer message between
/// threads
#[derive(Debug)]
pub(crate) struct MspcChannel<T> {
    /// the messages will be stored here
    cached_messages: Arc<Mutex<List>>,
    /// used for checking duplicate keys
    counter: Arc<RwLock<HashSet<String>>>,
    /// the capcity of a channel
    bounded_size: i32,
    /// use condVar to support block recieve
    cond_var_recieve: Arc<Condvar>,
    /// use condVar to support block send
    cond_var_send: Arc<Condvar>,
    /// global message_id
    message_id: Mutex<i32>,
    /// global map: id -> node
    id_to_node: CellMap<i32, Arc<Cell>>,
    /// global map: key -> [duplicate_message_id0,duplicate_messagey_id1,...]
    key_message_id_in_duplicate: CellMap<String, HashSet<i32>>,
    /// id to Message
    id_to_message: CellMap<i32, InternalMessage<T>>,
    /// golbal map: key -> [message_id0,message_id1,...]
    key_to_message_id: CellMap<String, HashSet<i32>>,
}

impl<T> MspcChannel<T> {
    /// check message is valid or not
    pub(crate) fn is_valid(&self, message_id: i32) -> bool {
        let read_guard = self.id_to_message.0.borrow();
        if let Some(res) = read_guard.get(&message_id) {
            let read_guard2 = res.ref_count.read();
            return *read_guard2 == 0;
        }
        false
    }

    /// remove a message
    pub(crate) fn remove(&self, message_id: i32) -> InternalMessage<T> {
        // get lock
        let mut list = self.cached_messages.lock();
        let mut id_to_message_guard = self.id_to_message.0.borrow_mut();
        let mut id_to_node_guard = self.id_to_node.0.borrow_mut();
        let res = id_to_message_guard.remove(&message_id);
        // remove message_id in list
        if let Some(node) = id_to_node_guard.remove(&message_id) {
            list.remove(&node);
        }
        if let Some(message_0) = res {
            let mut write_counter = self.counter.write();
            let mut write_message_id = self.key_to_message_id.0.borrow_mut();
            let mut write_message_id_duplicate = self.key_message_id_in_duplicate.0.borrow_mut();
            for i in 0..message_0.keys.len() {
                if let Some(key_) = message_0.keys.get(i) {
                    // update counter
                    let _ = write_counter.insert(String::from(&key_.0));
                    // get duplicate_set
                    let duplicate_set = write_message_id_duplicate
                        .entry(String::from(&key_.0))
                        .or_insert_with(HashSet::new);
                    //  update key_to_message_id
                    if let Some(set) = write_message_id.get_mut(&key_.0) {
                        let _ = set.remove(&message_id);
                        // update key_message_id_in_duplicate

                        for id in set.iter() {
                            let _ = duplicate_set.insert(*id);
                            let message_opt = id_to_message_guard.get(id);
                            if let Some(message) = message_opt {
                                // update ref_count
                                let mut ref_count_guard = message.ref_count.write();
                                *ref_count_guard = ref_count_guard.wrapping_add(1);
                                // change the list
                                if let Some(node) = id_to_node_guard.get(id) {
                                    list.remove(&Arc::<Cell>::clone(node));
                                    list.list_push_back(&Arc::<Cell>::clone(node));
                                }
                            }
                        }
                    }
                }
            }
            return message_0;
        }
        panic!("Can't be none")
    }

    /// `channel` func is used to get sender and reciever
    pub(crate) fn channel(bounded_size_: i32) -> (Sender<T>, Reciever<T>) {
        let message_channel = Arc::new(MspcChannel {
            cached_messages: Arc::new(Mutex::new(List::new())),
            counter: Arc::new(RwLock::new(HashSet::<String>::new())),
            bounded_size: bounded_size_,
            cond_var_recieve: Arc::new(Condvar::new()),
            cond_var_send: Arc::new(Condvar::new()),
            message_id: Mutex::new(0),
            id_to_node: CellMap(RefCell::new(HashMap::new())),
            key_message_id_in_duplicate: CellMap(RefCell::new(HashMap::new())),
            id_to_message: CellMap(RefCell::new(HashMap::new())),
            key_to_message_id: CellMap(RefCell::new(HashMap::new())),
        });
        let sender = Sender {
            chan: Arc::clone(&message_channel),
        };
        let reciever = Reciever {
            chan: Arc::clone(&message_channel),
        };
        (sender, reciever)
    }
    /// push message in channel
    pub(crate) fn push_message(&self, message: InternalMessage<T>) {
        let mut write_guard_0 = self.cached_messages.lock();
        let mut flag = false;
        {
            let read_guard = self.counter.read();
            let mut write_guard = self.key_message_id_in_duplicate.0.borrow_mut();
            let mut wirte_message = message.ref_count.write();
            let mut write_message2 = self.key_to_message_id.0.borrow_mut();
            for key_ in &message.keys {
                if write_message2.get(&key_.0).is_none() {
                    let _ = write_message2.insert(String::from(&key_.0), HashSet::new());
                }
                if let Some(set) = write_message2.get_mut(&key_.0) {
                    let _ = set.insert(message.id);
                }
                let res = read_guard.get(&key_.0);
                if res.is_some() {
                    if write_guard.get(&key_.0).is_none() {
                        let _ = write_guard.insert(String::from(&key_.0), HashSet::new());
                    }
                    if let Some(set) = write_guard.get_mut(&key_.0) {
                        let _ = set.insert(message.id);
                    }
                    *wirte_message = wirte_message.wrapping_add(1);
                    flag = true;
                }
            }
        }
        let item = ListNode::create_node(message.id);
        let mut write_guard2 = self.id_to_node.0.borrow_mut();
        let _ = write_guard2.insert(message.id, Arc::<Cell>::clone(&item));
        if flag {
            write_guard_0.list_push_back(&item);
        } else {
            write_guard_0.list_push_first(&item);
        }
        let mut write_guard_id_to_message = self.id_to_message.0.borrow_mut();
        let _ = write_guard_id_to_message.insert(message.id, message);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_me() {
        let (sender, reciever) = MspcChannel::<i32>::channel(3);
        let strs = vec![String::from("a"), String::from("b")];
        let message = InternalMessage::new(strs, 33);
        let res0 = sender.send(message);
        assert!(res0.is_ok());
        let res = reciever.recv();
        assert!(res.is_ok());
    }
}
