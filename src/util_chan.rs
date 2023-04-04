use std::marker::PhantomData;

use crate::channel::mspc_channel::{InternalMessage, MspcChannel, Reciever, Sender};
/// Chan is a wrapper for `mspc_channel`
#[derive(Debug)]
pub struct Chan<T> {
    /// a magic filed
    a: PhantomData<T>,
}

/// Message is a wrapper for `internal_message`
#[derive(Debug)]
pub struct Message<T> {
    /// a magic filed
    a: PhantomData<T>,
}

impl<T> Chan<T> {
    /// `create_chan func` is used to get sender and reciever to transfer
    /// message between chan
    #[inline]
    #[must_use]
    pub fn create_chan(bounded_size: i32) -> (Sender<T>, Reciever<T>) {
        MspcChannel::<T>::channel(bounded_size)
    }
}

impl<T> Message<T> {
    /// `create_internal_message` func is used to get `InternalMessage`
    #[inline]
    pub fn create_internal_message(vecs: Vec<String>, data: T) -> InternalMessage<T> {
        InternalMessage::new(vecs, data)
    }
}
