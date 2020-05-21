extern crate libq;

pub mod subscriber;
pub mod topic;
pub mod api;

use topic::{TopicState, Topic};
use std::collections::HashMap;
use api::ResponseType;
use std::thread;

pub struct Bus {
    topics: HashMap<String, Topic>,
    subscribers: HashMap<String, String> // Map from subscriber ids -> topic name
}

impl Bus {
    pub fn new() -> Self {
        return Bus{
            topics: HashMap::new(),
            subscribers: HashMap::new()
        };
    }

    pub fn create_topic(&mut self, topic: Topic) -> ResponseType {
        let topic_str = topic.name.clone();
        if self.topics.contains_key(&topic_str) {
            let mut existing_topic = self.topics.get_mut(&topic_str).unwrap();
            if existing_topic.state == TopicState::MarkedForDeletion {
                existing_topic.state = TopicState::MarkedForRecreation;
                return ResponseType::Ok;
            }
            return ResponseType::NothingHappened;
        }

        self.topics.insert(topic_str, topic);
        return ResponseType::Ok;
    }

    pub fn delete_topic(&mut self, topic: Topic) -> ResponseType {
        let topic_str = topic.name.clone();
        if self.topics.contains_key(&topic_str) {
            self.topics.remove(&topic_str);
            return ResponseType::Ok;
        }

        return ResponseType::NothingHappened;
    }

    pub fn create_subscription(&mut self, topic: Topic) -> Vec<u8> {
        let topic_str = topic.name.clone();

        if self.topics.contains_key(&topic_str) {
            let topic = self.topics.get_mut(&topic_str).unwrap();
            let sub = topic.add_subscriber();

            self.subscribers.insert(sub.id.clone(), topic_str);

            let mut response = vec![ResponseType::Ok.into(), 1, 4];

            response.append(&mut sub.id.clone().bytes().collect());

            return response;
        }

        return vec![ResponseType::DoesntExist.into()];
    }

    pub fn publish_message(&mut self, topic_name: &String, message: Vec<u8>) -> ResponseType {
        if self.topics.contains_key(topic_name) {
            let topic = self.topics.get_mut(topic_name).unwrap();
            topic.publish_message(message);
            return ResponseType::Ok;
        }

        return ResponseType::DoesntExist;
    }

    pub fn try_get_message(&mut self, subscriber_id: &String) -> Result<Option<Vec<u8>>, ResponseType> {
        if self.subscribers.contains_key(subscriber_id) {
            let topic_name = self.subscribers.get(subscriber_id).unwrap();
            let mut topic = self.topics.get_mut(topic_name).unwrap();

            match topic.try_get_message(subscriber_id) {
                Ok(maybe_msg) => {
                    match maybe_msg {
                        Some(msg) => {
                            return Ok(Some(msg));
                        },
                        None => {
                            return Ok(None);
                        }
                    }
                },
                Err(code) => {
                    return Err(code);
                }
            }
        }

        return Err(ResponseType::DoesntExist);
    }

    pub fn set_subcription_waiting(&mut self, subscriber_id: &String) {
        if self.subscribers.contains_key(subscriber_id) {
            let topic_name = self.subscribers.get(subscriber_id).unwrap();
            let mut topic = self.topics.get_mut(topic_name).unwrap();
            topic.waiting_threads.push(thread::current());
        }
    }
}

