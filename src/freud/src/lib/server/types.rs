use std::convert::TryFrom;
use std::string::FromUtf8Error;
use std::thread::{self, Thread};

use super::api::{FreudianTopicRequest, FreudianSubscriptionRequest};

use libq::rand;

#[derive(Debug, PartialEq)]
pub enum FreudianError {
    InvalidString(Vec<u8>),
    TopicDoesntExist,
    TopicAlreadyExists,
    SubscriptionDoesntExist,
    NoSubscribers,
    NoNewMessages,
    InvalidResponse
}

#[derive(Debug, PartialEq)]
pub enum FreudianResponse {
    Empty,
    Subscription(UUID),
    Message(Vec<u8>)
}

impl From<FromUtf8Error> for FreudianError {
    fn from(err: FromUtf8Error) -> FreudianError {
        return FreudianError::InvalidString(err.into_bytes());
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct UUID {
    pub uuid: [u8;16],
}

impl UUID {
    fn random() -> UUID {
        let mut bytes = [0; 16];
        rand::fill_exact(&mut bytes).expect("Failed to read random bytes");
        return UUID{
            uuid: bytes
        }
    }
}

pub struct Subscription {
    pub uuid: UUID,
    current_offset: u64
}

impl From<FreudianSubscriptionRequest> for Subscription {
    fn from(req: FreudianSubscriptionRequest) -> Subscription {
        return Subscription{
            uuid: UUID {
                uuid: req.subscription_id
            },
            current_offset: 0
        }
    }
}

impl PartialEq for Subscription {
    fn eq(&self, other: &Self) -> bool {
        return self.uuid == other.uuid;
    }
}

impl Subscription {
    pub fn new(offset: u64) -> Subscription {
        return Subscription {
            uuid: UUID::random(),
            current_offset: offset
        }
    }
}

pub struct Message {
    offset: u64,
    contents: Vec<u8>
}

impl Message {
    fn new(contents: Vec<u8>, offset: u64) -> Message {
        return Message {
            offset: offset,
            contents: contents
        }
    }
}

pub struct Topic {
    pub name: String,
    pub latest_offset: u64,
    subscribers: Vec<Subscription>,
    messages: Vec<Message>,
    waiting_threads: Vec<Thread>
}

impl Topic {
    pub fn new(name: String) -> Topic{
        return Topic {
            name: name,
            latest_offset: 0,
            subscribers: Vec::new(),
            messages: Vec::new(),
            waiting_threads: Vec::new()
        }
    }

    pub fn set_thread_waiting(&mut self) {
        self.waiting_threads.push(thread::current());
    }

    pub fn add_subscriber(&mut self, sub: Subscription) {
        self.subscribers.push(sub);
    }

    pub fn remove_subscriber(&mut self, sub: &Subscription) {
        self.subscribers.retain(|s| s != sub);
    }

    pub fn has_subscription(&self, sub: &Subscription) -> bool {
        return self.subscribers.iter().find(|&s| s == sub).is_some();
    }

    pub fn add_message(&mut self, message: Vec<u8>) -> Result<FreudianResponse, FreudianError> {
        if self.subscribers.len() == 0 {
            return Err(FreudianError::NoSubscribers);
        }
        self.messages.push(Message::new(message, self.latest_offset));
        self.latest_offset += 1;

        // Unpark all the threads waiting for a message
        for thread in self.waiting_threads.iter() {
            thread.unpark();
        }

        self.waiting_threads.clear();

        return Ok(FreudianResponse::Empty);
    }

    pub fn get_message(&mut self, sub: &Subscription) -> Result<FreudianResponse, FreudianError> {
        let mut actual_sub = match self.subscribers.iter_mut().find(|s| &s == &&sub) {
            Some(sub) => sub,
            None => {
                return Err(FreudianError::SubscriptionDoesntExist);
            }
        };

        let new_message = match self.messages.iter().find(|&msg| msg.offset >= actual_sub.current_offset) {
            Some(msg) => msg,
            None => {
                return Err(FreudianError::NoNewMessages);
            }
        };

        actual_sub.current_offset = new_message.offset + 1;

        return Ok(FreudianResponse::Message(new_message.contents.clone()));
    }
}

impl TryFrom<FreudianTopicRequest> for Topic {
    type Error = FreudianError;
    fn try_from(req: FreudianTopicRequest) -> Result<Self, Self::Error> {
        return Ok(Topic::new(String::from_utf8(req.topic_name)?));
    }
}

impl PartialEq for Topic {
    fn eq(&self, other: &Self) -> bool {
        return self.name == other.name;
    }
}
