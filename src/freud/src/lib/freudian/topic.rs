use super::subscriber::Subscription;
use std::collections::HashMap;
use super::api::ResponseType;
use std::thread;
use std::fmt;

#[derive(PartialEq)]
pub enum TopicState {
    Steady,
    MarkedForDeletion,
    MarkedForRecreation
}

#[derive(PartialEq)]
#[derive(Debug)]
pub struct Message {
    offset: u64,
    message: Vec<u8>,
    to_read_count: u32 // Count of subscribers yet to read this message
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return writeln!(f, "Message:<ID: {}, Contents: [{}], Still to Read: {}>", self.offset, self.message.iter().map(|m| m.to_string()).collect::<Vec<String>>().join(", "), self.to_read_count);
    }
}

pub struct Topic {
    pub name: String,
    pub state: TopicState,
    pub subscribers: HashMap<String, Subscription>,
    messages: Vec<Message>,
    pub waiting_threads: Vec<thread::Thread>,
    current_offset: u64,
}

impl Topic {
    pub fn new(name: String) -> Topic {
        return Topic {
            name: name,
            state: TopicState::Steady,
            subscribers: HashMap::new(),
            messages: Vec::new(),
            waiting_threads: Vec::new(),
            current_offset: 1
        };
    }

    pub fn add_subscriber(&mut self) -> &Subscription {
        let sub = Subscription::new().unwrap();
        let id = sub.id.clone();
        self.subscribers.insert(sub.id.clone(), sub);
        return self.subscribers.get(&id).unwrap();
    }

    pub fn publish_message(&mut self, message: Vec<u8>) {
        if self.subscribers.len() == 0 {
            // If we don't have any subscribers, just drop the message
            return;
        }

        let new_offset = self.current_offset;
        self.current_offset += 1; // This is thread safe because we assume we're in the context of a locked Bus
        let new_msg = Message{
            offset: new_offset,
            message: message,
            to_read_count: self.subscribers.len() as u32
        };

        self.messages.push(new_msg);

        // Unpark all the connections waiting for this to get new messages
        for thread in self.waiting_threads.iter() {
            thread.unpark();
        }
        self.waiting_threads.clear();
    }

    pub fn try_get_message(&mut self, subcription_id: &String) -> Result<Option<Vec<u8>>, ResponseType> {
        if self.subscribers.contains_key(subcription_id) {
            let subscriber = self.subscribers.get_mut(subcription_id).unwrap();
            match self.messages.iter_mut().filter(|x| x.offset > subscriber.offset).next() {
                Some(message) => {
                    subscriber.offset = message.offset;
                    message.to_read_count -= 1;
                    if message.to_read_count == 0 {
                        // TODO: Remove from messages self.messages.remove_item(message);
                    }
                    return Ok(Some(message.message.clone()));
                },
                None => {
                    return Ok(None);
                }
            }
        }

        return Err(ResponseType::DoesntExist);
    }
}
