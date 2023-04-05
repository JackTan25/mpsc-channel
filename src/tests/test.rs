#[cfg(test)]
pub(crate) mod test_channel {
    use crate::{channel::mspc_channel::*, errors::Errors};
    use std::sync::Arc;
    #[test]
    fn test_basic_channel() {
        let (sender, reciever) = MspcChannel::<i32>::channel(-1);
        let strs = vec![String::from("a"), String::from("b")];
        let message = InternalMessage::new(strs, 33);
        let res0 = sender.send(message);
        assert!(res0.is_ok());
        let res = reciever.recv();
        assert!(res.is_ok());
        let data = res.unwrap();
        assert_eq!(data.data, 33);
    }

    #[test]
    fn test_keys_duplicate() {
        let (sender0, reciever) = MspcChannel::<i32>::channel(-1);
        let sender = Arc::new(sender0);
        let mut hanlders = Vec::new();
        for i in 1..=4 {
            let shared_sender = sender.clone();
            let handler = std::thread::spawn(move || {
                let sender_local = shared_sender.clone();
                let strs = vec![String::from("a"), String::from("b")];
                let message = InternalMessage::new(strs, i);
                let res = sender_local.send(message);
                assert!(res.is_ok())
            });
            hanlders.push(handler)
        }
        for handler in hanlders {
            handler.join().unwrap();
        }
        // for now we have all data in channel
        {
            let message = reciever.recv();
            assert!(message.is_ok());
            // message 'Active'
            assert!(reciever.recv().is_err());
            if let Err(err) = reciever.recv() {
                assert_eq!(err, Errors::KeyDuplicate);
            }
            // message drop here
        }
        {
            let message = reciever.recv();
            assert!(message.is_ok());
            // message 'Active'
            assert!(reciever.recv().is_err());
            if let Err(err) = reciever.recv() {
                assert_eq!(err, Errors::KeyDuplicate);
            }
            // message drop here
        }
        {
            let message = reciever.recv();
            assert!(message.is_ok());
            // message 'Active'
            assert!(reciever.recv().is_err());
            if let Err(err) = reciever.recv() {
                assert_eq!(err, Errors::KeyDuplicate);
            }
            // message drop here
        }
        {
            let message = reciever.recv();
            assert!(message.is_ok());
            // the chan is empty now, can't use recv, otherwise it will
            // block here, this is not convient for this test.
        }
    }

    #[test]
    fn test_keys_without_duplicate() {
        let (sender0, reciever) = MspcChannel::<i32>::channel(-1);
        let sender = Arc::new(sender0);
        let mut hanlders = Vec::new();
        for i in 1..=4 {
            let shared_sender = sender.clone();
            let handler = std::thread::spawn(move || {
                let sender_local = shared_sender.clone();
                let str = std::format!("{}", i);
                let strs = vec![String::from("a") + &str, String::from("b") + &str];
                let message = InternalMessage::new(strs, i);
                let res = sender_local.send(message);
                assert!(res.is_ok())
            });
            hanlders.push(handler)
        }
        for handler in hanlders {
            handler.join().unwrap();
        }
        // for now we have all data in channel
        let message = reciever.recv();
        assert!(message.is_ok());
        let message = reciever.recv();
        assert!(message.is_ok());
        let message = reciever.recv();
        assert!(message.is_ok());
        let message = reciever.recv();
        assert!(message.is_ok());
    }
    #[test]
    fn test_get_valid_message_first() {
        let (sender0, reciever) = MspcChannel::<i32>::channel(-1);
        let sender = Arc::new(sender0);
        let mut hanlders = Vec::new();
        for i in 1..=4 {
            let shared_sender = sender.clone();
            let handler = std::thread::spawn(move || {
                let sender_local = shared_sender.clone();
                let str = std::format!("{}", i);
                let strs = vec![String::from("a") + &str, String::from("b") + &str];
                let message = InternalMessage::new(strs, i);
                let res = sender_local.send(message);
                assert!(res.is_ok())
            });
            hanlders.push(handler)
        }
        for i in 1..=4 {
            let shared_sender = sender.clone();
            let handler = std::thread::spawn(move || {
                let sender_local = shared_sender.clone();
                let str = std::format!("{}", i);
                let strs = vec![String::from("a") + &str, String::from("b") + &str];
                let message = InternalMessage::new(strs, i);
                let res = sender_local.send(message);
                assert!(res.is_ok())
            });
            hanlders.push(handler)
        }
        for handler in hanlders {
            handler.join().unwrap();
        }
        // for now we have all data in channel
        {
            let message0 = reciever.recv();
            assert!(message0.is_ok());
            let message1 = reciever.recv();
            assert!(message1.is_ok());
            let message2 = reciever.recv();
            assert!(message2.is_ok());
            let message3 = reciever.recv();
            assert!(message3.is_ok());
            // message 'Active'
            assert!(reciever.recv().is_err());
            if let Err(err) = reciever.recv() {
                assert_eq!(err, Errors::KeyDuplicate);
            }
            // messages drop here
        }
        {
            let message0 = reciever.recv();
            assert!(message0.is_ok());
            let message1 = reciever.recv();
            assert!(message1.is_ok());
            let message2 = reciever.recv();
            assert!(message2.is_ok());
            let message3 = reciever.recv();
            assert!(message3.is_ok());
            // messages drop here
        }
    }
}
