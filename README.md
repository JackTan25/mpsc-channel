```rs
    This project create a new Mpsc(multi producer and single consumer) channel.
The design doc is here:
```
Message Struct Design:
```rs
pub struct InternalMessage<T> {
    /// a message with multi-keys
    keys: Vec<Key<T>>,
    /// a message data
    pub data: T,
}

/// Key is a struct type, we use it as the
/// message's key
#[derive(Debug)]
pub(crate) struct Key<T>(String, Option<Arc<MspcChannel<T>>>);
```
Channel Design
```rs
pub(crate) struct MspcChannel<T> {
    /// the messages will be stored here
    cached_messages: Arc<RwLock<Vec<InternalMessage<T>>>>,
    /// used for checking duplicate keys
    counter: Arc<RwLock<HashSet<String>>>,
    /// the capcity of a channel
    bounded_size: i32,
}
```
Send Message and Recieve Message
```rs
// it support clone(), so that support
// multi-producer.
pub struct Sender<T> {
    /// hold a channel ref_counter
    chan: Arc<MspcChannel<T>>,
}

// use deny to stop the clone, we
// make sure the single customer.
#[deny(clippy::clone_on_ref_ptr)]
pub struct Reciever<T> {
    /// hold a channel ref_counter
    chan: Arc<MspcChannel<T>>,
}

    Note that: we will store a Arc<MspcChannel<T>> in reciever and sender,
so that they can call the channel to get or send message. every time when you
recieve message from channel,  we will use the counter to test the 'Active'.
We re-implement the drop trait for Key<T>, when it leaves its lifetime, we will
change the counter of the channel to make it 'dead' not 'Active'. And counter is 
used to check the duplicat-keys for the messages.
```