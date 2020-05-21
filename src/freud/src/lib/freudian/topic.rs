use super::subscriber::Subscription;
use std::collections::HashMap;
use std::thread;
use std::time::Duration;
use super::api::ResponseType;

#[derive(PartialEq)]
pub enum TopicState {
    Steady,
    MarkedForDeletion,
    MarkedForRecreation
}

#[derive(PartialEq)]
pub struct Message {
    offset: u64,
    message: Vec<u8>,
    to_read_count: u32 // Count of subscribers yet to read this message
}

pub struct Topic {
    pub name: String,
    pub state: TopicState,
    pub subscribers: HashMap<String, Subscription>,
    messages: Vec<Message>,
    waiting_threads: Vec<thread::Thread>,
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
            current_offset: 0
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
                        return Ok(Some(message.message.clone()));
                    }
                },
                None => return Ok(None)
            }
        }

        return Err(ResponseType::DoesntExist);
    }

    pub fn get_message(&mut self, subcription_id: &String, block_timeout_secs: i32) -> Result<Option<Vec<u8>>, ResponseType> {
        let mut timed_out = false;
        loop {
            match self.try_get_message(subcription_id) {
                Err(code) => return Err(code),
                Ok(maybe_msg) => {
                    match maybe_msg {
                        Some(msg) => return Ok(Some(msg)),
                        None => {
                            self.waiting_threads.push(thread::current());
                            if block_timeout_secs < 0 {
                                thread::park();
                            }
                            else {
                                if timed_out {
                                    // We're in the second iteration, after having timed out from the first
                                    return Ok(None);
                                }
                                thread::park_timeout(Duration::from_secs(block_timeout_secs as u64));
                                timed_out = true;
                            }
                        }
                    }
                }
            }
        }
    }
}
