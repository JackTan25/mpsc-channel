use crate::errors::{Errors, Result};
use log::error;
use parking_lot::{Mutex, RwLock};
use std::{collections::HashSet, sync::Arc};
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
}

impl<T> InternalMessage<T> {
    /// `new` is used to generate a `InternalMessage`
    pub fn new(vecs: Vec<String>, data_: T) -> InternalMessage<T> {
        let mut message = InternalMessage {
            keys: Vec::<Key<T>>::new(),
            data: data_,
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
        // -1 means this is a unbounded channel
        if self.chan.bounded_size == -1 {
            let mut write_guard = self.chan.cached_messages.lock();
            write_guard.push(message);
            Ok(())
        } else {
            loop {
                let mut write_guard = self.chan.cached_messages.lock();
                let bounded_size: usize = match self.chan.bounded_size.try_into() {
                    Err(e) => {
                        error!("type conversion error,{}", e);
                        return Err(Errors::TypeConversionError);
                    }
                    Ok(size) => size,
                };

                if write_guard.len() < bounded_size {
                    write_guard.push(message);
                    return Ok(());
                }
                drop(write_guard);
                // use yield_now() to optimize, otherwise
                // it will loop forever, cost too much computation
                // source.
                std::thread::yield_now();
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

impl<T> Reciever<T> {
    /// `recv` recieve message from channel
    pub fn recv(&self) -> Result<InternalMessage<T>> {
        // get write_guard
        loop {
            let mut write_guard = self.chan.cached_messages.lock();
            // 1.there is no message in channel
            // just loop ahead
            if write_guard.len() == 0 {
                drop(write_guard);
                // use yield_now() to optimize, otherwise
                // it will loop forever, cost too much computation
                // source.
                std::thread::yield_now();
                continue;
            }
            // we need to see all messages, if there is anyone message
            // is ok, just give it out.
            for i in 0..write_guard.len() {
                let opt = write_guard.get(i);
                if let Some(message) = opt {
                    // 2.test whether there exists duplicate key which is 'Active'
                    let flag = self.chan.duplicate_key(message);
                    if flag {
                        continue;
                    }
                    // add counter
                    let mut write_guard2 = self.chan.counter.write();
                    for key_ in &message.keys {
                        let _ = write_guard2.insert(String::from(&key_.0));
                    }
                    return Ok(write_guard.remove(0));
                }
                panic!("MessageOptError")
            }
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
    cached_messages: Arc<Mutex<Vec<InternalMessage<T>>>>,
    /// used for checking duplicate keys
    counter: Arc<RwLock<HashSet<String>>>,
    /// the capcity of a channel
    bounded_size: i32,
}

impl<T> MspcChannel<T> {
    /// `channel` func is used to get sender and reciever
    pub(crate) fn channel(bounded_size_: i32) -> (Sender<T>, Reciever<T>) {
        let message_channel = Arc::new(MspcChannel {
            cached_messages: Arc::new(Mutex::new(Vec::<InternalMessage<T>>::new())),
            counter: Arc::new(RwLock::new(HashSet::<String>::new())),
            bounded_size: bounded_size_,
        });
        let sender = Sender {
            chan: Arc::clone(&message_channel),
        };
        let reciever = Reciever {
            chan: Arc::clone(&message_channel),
        };
        (sender, reciever)
    }

    /// `duplicate_key` is used to test whether the message is 'Active'
    pub(crate) fn duplicate_key(&self, message: &InternalMessage<T>) -> bool {
        let read_guard = self.counter.read();
        for key_ in &message.keys {
            let res = read_guard.get(&key_.0);
            if res.is_some() {
                return true;
            }
        }
        false
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
