use libfreudian::Bus;
use libfreudian::api::{MessageType, ResponseType, TopicRequest, PutMessageRequest};

use std::sync::{Arc, Mutex};

pub fn handle_topic_request(bus: &mut Arc<Mutex<Bus>>, req: Option<TopicRequest>) -> Result<Vec<u8>, ()>{
    if req.is_some() {
        let req = req.unwrap();
        let locked_bus = bus.lock();
        if locked_bus.is_err(){
            return Err(());
        }

        let mut locked_bus = locked_bus.unwrap();

        return Ok(match req.class {
            MessageType::CreateTopic => vec![(*locked_bus).create_topic(req.into_topic()).into()],
            MessageType::DeleteTopic => vec![(*locked_bus).delete_topic(req.into_topic()).into()],
            MessageType::Subscribe => (*locked_bus).create_subscription(req.into_topic()),
            _ => vec![ResponseType::MalformedRequest.into()]
        });
    }
    return Ok(vec![ResponseType::MalformedRequest.into()]);
}

pub fn handle_add_message(bus: &mut Arc<Mutex<Bus>>, req: Option<PutMessageRequest>) -> Result<Vec<u8>, ()>{
    if req.is_some() {
        let req = req.unwrap();
        let locked_bus = bus.lock();
        if locked_bus.is_err(){
            return Err(());
        }

        let mut locked_bus = locked_bus.unwrap();
        if req.class == MessageType::ProduceMessage {
            return Ok(vec![(*locked_bus).publish_message(&req.topic_id, req.message).into()]);
        }
    }
    return Ok(vec![ResponseType::MalformedRequest.into()]);
}