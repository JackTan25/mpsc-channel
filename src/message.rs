// Key is a struct type, we use it as the
// message's key
#[derive(Debug)]
pub(crate) struct Key<T>(String,Option<Arc<MspcChannel<T>>>);

impl<T> Drop for Key<T>{
    fn drop(&mut self) {
        // get write_guard
        if let Some(channel) = &self.1{
        let mut write_guard = channel.counter.write();
        // write_guard
        let _ = write_guard.remove(&self.0);
        }
    }
}

// Message is a struct which is used 
// to passed by channel
#[derive(Debug)]
pub struct Message<T>{
    keys:Vec<Key<T>>,
    pub data:T,
}

impl<T> Message<T>{
    pub fn new(vecs:Vec<String>,data:T) -> Message<T>{
        let mut message = Message{
            keys:Vec::<Key<T>>::new(),
            data:data,
        };
        for vec in vecs{
            message.keys.push(Key(vec, None))
        }
        return  message;
    }
}