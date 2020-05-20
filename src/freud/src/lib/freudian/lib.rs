extern crate libq;

pub mod subscriber;
pub mod topic;
pub mod api;

use subscriber::Subscription;
use topic::{TopicState, Topic};
use std::collections::HashMap;
use api::ResponseType;

pub struct Bus {
    topics: HashMap<String, Topic>,
    subscribers: HashMap<String, String>
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
            println!("{}", &sub.id);
            let mut response = vec![ResponseType::Ok.into(), 1, 4];

            response.append(&mut sub.id.clone().bytes().collect());

            return response;
        }

        return vec![ResponseType::DoesntExist.into()];
    }
}

