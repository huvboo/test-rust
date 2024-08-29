use std::{any::Any, time::Duration};

use crossbeam_channel::{Receiver, Sender};

pub struct Message<T> {
    message_id: MessageId,
    data: T,
}

impl<T> Message<T> {
    pub fn new(message_id: MessageId, data: T) -> Message<T> {
        Message { message_id, data }
    }
}

pub struct DynamicMessage {
    pub message_id: MessageId,
    pub data: Box<dyn Any + Send>,
}

impl DynamicMessage {
    pub fn new(message_id: MessageId, data: impl Any + Send + 'static) -> DynamicMessage {
        DynamicMessage {
            message_id,
            data: Box::new(data),
        }
    }
}

pub fn send_message<T: Any + Send + 'static>(
    tx: Sender<DynamicMessage>,
    message_id: MessageId,
    data: T,
) {
    let msg = DynamicMessage::new(message_id, data);
    tx.send(msg).unwrap()
}

#[derive(Debug)]
pub enum MessageId {
    DroppedFile,
    CursorMoved,
    MouseWheel,
    Translate,
    Escape,
    Resized,
    RedrawRequested,
}

// impl<T: fmt::Display> fmt::Display for Message<T> {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(f, "Message ID: {}, Data: {}", self.message_id, self.data)
//     }
// }
