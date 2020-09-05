use super::api::{FreudianProduceMessageRequest, FreudianSubscriptionRequest, FreudianTopicRequest};
use super::types::{FreudianError, FreudianResponse, Subscription, Topic};
use std::convert::TryFrom;

use libq::io::{write_u32, Endianness};

pub struct FreudianBus {
    topics: Vec<Topic>,
}

impl FreudianBus {
    pub fn new() -> FreudianBus {
        return FreudianBus { topics: Vec::new() };
    }

    pub fn get_num_subscribers(&self, topic_req: FreudianTopicRequest) -> Result<FreudianResponse, FreudianError> {
        let test_topic = Topic::try_from(topic_req)?;

        if let Some(topic) = self.topics.iter().find(|&t| t == &test_topic) {
            let mut bytes = Vec::new();
            write_u32(&mut bytes, topic.num_subscribers(), &Endianness::Little).expect("Failed to write into vector");
            return Ok(FreudianResponse::Message(bytes));
        } else {
            return Err(FreudianError::TopicDoesntExist);
        }
    }

    pub fn get_topics(&self) -> Result<FreudianResponse, FreudianError> {
        let mut bytes = Vec::new();

        write_u32(&mut bytes, self.topics.len() as u32, &Endianness::Little).expect("Failed to write into vector");

        for topic in self.topics.iter() {
            let mut title_bytes = topic.name.bytes().collect::<Vec<u8>>();
            write_u32(&mut bytes, title_bytes.len() as u32, &Endianness::Little).expect("Failed to write into vector");
            bytes.append(&mut title_bytes);
        }

        return Ok(FreudianResponse::Message(bytes));
    }

    pub fn create_topic(&mut self, topic_req: FreudianTopicRequest) -> Result<FreudianResponse, FreudianError> {
        let new_topic = Topic::try_from(topic_req)?;

        if self.topics.iter().find(|&t| t == &new_topic).is_some() {
            return Err(FreudianError::TopicAlreadyExists);
        }

        self.topics.push(new_topic);

        return Ok(FreudianResponse::Empty);
    }

    pub fn delete_topic(&mut self, topic_req: FreudianTopicRequest) -> Result<FreudianResponse, FreudianError> {
        let to_delete_topic = Topic::try_from(topic_req)?;
        let mut did_delete = false;

        self.topics.retain(|val| {
            let to_be_deleted = val == &to_delete_topic;
            did_delete |= to_be_deleted;

            return !to_be_deleted;
        });

        if !did_delete {
            return Err(FreudianError::TopicDoesntExist);
        }

        return Ok(FreudianResponse::Empty);
    }

    pub fn subscribe(&mut self, topic_req: FreudianTopicRequest) -> Result<FreudianResponse, FreudianError> {
        let to_subscribe_topic = Topic::try_from(topic_req)?;

        let actual_topic = match self.topics.iter_mut().find(|topic| topic == &&to_subscribe_topic) {
            Some(topic) => topic,
            None => {
                // If the topic doesn't exist, bail
                return Err(FreudianError::TopicDoesntExist);
            }
        };

        let subscription = Subscription::new(actual_topic.latest_offset);
        let uuid = subscription.uuid.clone();
        actual_topic.add_subscriber(subscription);

        return Ok(FreudianResponse::Subscription(uuid));
    }

    pub fn unsubscribe(&mut self, subscription_req: FreudianSubscriptionRequest) -> Result<FreudianResponse, FreudianError> {
        let subscription = Subscription::from(subscription_req);
        let topic = match self.topics.iter_mut().find(|topic| topic.has_subscription(&subscription)) {
            Some(topic) => topic,
            None => {
                return Err(FreudianError::SubscriptionDoesntExist);
            }
        };

        topic.remove_subscriber(&subscription);

        return Ok(FreudianResponse::Empty);
    }

    pub fn produce_message(&mut self, message_req: FreudianProduceMessageRequest) -> Result<FreudianResponse, FreudianError> {
        let to_add_topic = Topic::try_from(message_req.topic_request)?;

        let actual_topic = match self.topics.iter_mut().find(|topic| topic == &&to_add_topic) {
            Some(topic) => topic,
            None => {
                // If the topic doesn't exist, bail
                return Err(FreudianError::TopicDoesntExist);
            }
        };

        return actual_topic.add_message(message_req.message);
    }

    pub fn consume_message(&mut self, subscription_req: FreudianSubscriptionRequest) -> Result<FreudianResponse, FreudianError> {
        let subscription = Subscription::from(subscription_req);
        let topic = match self.topics.iter_mut().find(|topic| topic.has_subscription(&subscription)) {
            Some(topic) => topic,
            None => {
                return Err(FreudianError::SubscriptionDoesntExist);
            }
        };

        return topic.get_message(&subscription);
    }

    pub fn mark_thread_waiting(&mut self, subscription_req: FreudianSubscriptionRequest) -> Result<(), FreudianError> {
        let subscription = Subscription::from(subscription_req);
        let topic = match self.topics.iter_mut().find(|topic| topic.has_subscription(&subscription)) {
            Some(topic) => topic,
            None => {
                return Err(FreudianError::SubscriptionDoesntExist);
            }
        };

        topic.set_thread_waiting();

        return Ok(());
    }
}
