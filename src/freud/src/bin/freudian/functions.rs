use libfreudian::Bus;
use libfreudian::api::{MessageType, ResponseType, OneValueRequest, PutMessageRequest};
use libfreudian::util::make_response;
use std::time::Duration;
use std::thread;

use std::sync::{Arc, Mutex};

pub fn handle_topic_request(bus: &mut Arc<Mutex<Bus>>, req: Option<OneValueRequest>) -> Result<Vec<u8>, ()>{
    if req.is_some() {
        let req = req.unwrap();
        let locked_bus = bus.lock();
        if locked_bus.is_err(){
            return Err(());
        }

        let mut locked_bus = locked_bus.unwrap();

        return Ok(match req.class {
            MessageType::CreateTopic => (*locked_bus).create_topic(req.into_topic()),
            MessageType::DeleteTopic => (*locked_bus).delete_topic(req.into_topic()),
            MessageType::Subscribe => (*locked_bus).create_subscription(req.into_topic()),
            _ => make_response(ResponseType::MalformedRequest, None)
        });
    }
    return Ok(make_response(ResponseType::MalformedRequest, None));
}

pub fn handle_get_message_request(bus: &mut Arc<Mutex<Bus>>, req: Option<OneValueRequest>) -> Result<Vec<u8>, ()>{
    if !req.is_some() {
        return Ok(make_response(ResponseType::MalformedRequest, None));
    }
    let req = req.unwrap();
    let mut timed_out = false;
    let block_timeout_secs = -1;
    loop {
        let maybe_message;
        {
            let locked_bus = bus.lock();
            if locked_bus.is_err(){
                return Err(());
            }

            let mut locked_bus = locked_bus.unwrap();
            maybe_message = locked_bus.try_get_message(&req.value);
        }
        match maybe_message {
            Err(code) => {
                return Ok(make_response(code, None));
            },
            Ok(maybe_msg) => {
                match maybe_msg {
                    Some(msg) => {
                        return Ok(make_response(ResponseType::Ok, Some(msg)));
                    },
                    None => {
                        {
                            let locked_bus = bus.lock();
                            if locked_bus.is_err(){
                                return Err(());
                            }

                            let mut locked_bus = locked_bus.unwrap();
                            locked_bus.set_subcription_waiting(&req.value);
                        }
                        if block_timeout_secs < 0 {
                            thread::park();
                        }
                        else {
                            if timed_out {
                                // We're in the second iteration, after having timed out from the first
                                return Ok(make_response(ResponseType::DoesntExist, None));
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

pub fn handle_add_message(bus: &mut Arc<Mutex<Bus>>, req: Option<PutMessageRequest>) -> Result<Vec<u8>, ()>{
    if req.is_some() {
        let req = req.unwrap();
        let locked_bus = bus.lock();
        if locked_bus.is_err(){
            return Err(());
        }

        let mut locked_bus = locked_bus.unwrap();
        if req.class == MessageType::ProduceMessage {
            return Ok((*locked_bus).publish_message(&req.topic_id, req.message));
        }
    }
    return Ok(make_response(ResponseType::MalformedRequest, None));
}