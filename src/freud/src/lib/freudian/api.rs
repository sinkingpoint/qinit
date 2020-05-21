use std::convert::From;
use super::topic::Topic;

#[derive(Debug)]
#[derive(PartialEq)]
pub enum MessageType {
    CreateTopic,
    DeleteTopic,
    Subscribe,
    Unsubscribe,
    ProduceMessage,
    GetMessage,
    Unknown,
    Invalid
}

impl From<u8> for MessageType {
    fn from(id: u8) -> Self {
        match id {
            0 => MessageType::CreateTopic,
            1 => MessageType::DeleteTopic,
            2 => MessageType::Subscribe,
            3 => MessageType::Unsubscribe,
            4 => MessageType::ProduceMessage,
            5 => MessageType::GetMessage,
            _ => MessageType::Unknown
        }
    }
}

impl Into<u8> for MessageType {
    fn into(self) -> u8 {
        match self {
            MessageType::CreateTopic => 0,
            MessageType::DeleteTopic => 1,
            MessageType::Subscribe => 2,
            MessageType::Unsubscribe => 3,
            MessageType::ProduceMessage => 4,
            MessageType::GetMessage => 4,
            _ => 255
        }
    }
}

pub enum ResponseType {
    Ok,
    NothingHappened,
    No,
    DoesntExist,
    MalformedRequest
}

impl From<u8> for ResponseType {
    fn from(id: u8) -> Self {
        match id {
            0 => ResponseType::Ok,
            1 => ResponseType::NothingHappened,
            2 => ResponseType::No,
            3 => ResponseType::DoesntExist,
            _ => ResponseType::MalformedRequest
        }
    }
}

impl Into<u8> for ResponseType {
    fn into(self) -> u8 {
        match self {
            ResponseType::Ok => 0,
            ResponseType::NothingHappened => 1,
            ResponseType::No => 2,
            ResponseType::DoesntExist => 3,
            ResponseType::MalformedRequest => 4
        }
    }
}

/// TopicRequest is a catch all for requests that just take a topic name
pub struct TopicRequest {
    pub class: MessageType,
    pub topic_name: String,
}

impl TopicRequest {
    fn new(class: MessageType, name: String) -> TopicRequest {
        return TopicRequest{
            class: class,
            topic_name: name
        };
    }

    fn new_broken() -> TopicRequest {
        return TopicRequest{
            class: MessageType::Invalid,
            topic_name: String::new()
        }
    }

    pub fn into_bytes(mut self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();
        bytes.push(self.class.into());

        if self.topic_name.len() >= 2 << 16 {
            bytes.append(&mut [255, 255].into_iter().copied().collect());
        }
        else{
            let length = self.topic_name.len() as u16;
            bytes.append(&mut [(length & 0xFF00 >> 8) as u8, (length & 0xFF) as u8].into_iter().copied().collect());
        }

        self.topic_name.truncate(2<<16-1);
        bytes.append(&mut self.topic_name.into_bytes());
        return bytes;
    }

    pub fn into_topic(self) -> Topic {
        return Topic::new(self.topic_name);
    }
}

impl From<&Vec<u8>> for TopicRequest {
    fn from(raw: &Vec<u8>) -> Self {
        if raw.len() < 2 {
            // Minimum size is 2 bytes for topic name size
            return TopicRequest::new_broken();
        }
        let mut bytes = raw.into_iter();

        let topic_name_length = ((*bytes.next().unwrap() as u16) << 8 | (*bytes.next().unwrap() as u16)) as usize;
        if raw.len() - 2 != topic_name_length {
            eprintln!("Failed to parse into TopicRequest: Topic length {}, but only {} bytes left", topic_name_length, raw.len()-2);
            // Rest of the bytes are the wrong length
            return TopicRequest::new_broken();
        }

        let topic_name = match String::from_utf8(bytes.map(|b| *b).collect()) {
            Err(_err) => {
                eprintln!("Failed to parse into TopicRequest: TopicName is not utf8");
                // Rest of the bytes are invalid utf-8
                return TopicRequest::new_broken();
            },
            Ok(s) => s
        };

        return TopicRequest::new(MessageType::Unknown, String::from(topic_name));
    }
}

pub fn parse_as_create_topic_request(raw: &Vec<u8>) -> Option<TopicRequest> {
    let mut request = TopicRequest::from(raw);
    if request.class == MessageType::Invalid {
        return None;
    }
    request.class = MessageType::CreateTopic;
    return Some(request);
}

pub fn parse_as_delete_topic_request(raw: &Vec<u8>) -> Option<TopicRequest> {
    let mut request = TopicRequest::from(raw);
    if request.class == MessageType::Invalid {
        return None;
    }
    request.class = MessageType::DeleteTopic;
    return Some(request);
}

pub fn parse_as_subscribe_request(raw: &Vec<u8>) -> Option<TopicRequest> {
    let mut request = TopicRequest::from(raw);
    if request.class == MessageType::Invalid {
        return None;
    }
    request.class = MessageType::Subscribe;
    return Some(request);
}

pub struct EmptyRequest {
    pub class: MessageType
}

impl EmptyRequest {
    fn new(class: MessageType) -> EmptyRequest {
        return EmptyRequest{
            class: class
        };
    }

    fn new_broken() -> EmptyRequest {
        return EmptyRequest{
            class: MessageType::Invalid
        }
    }

    pub fn into_bytes(self) -> Vec<u8> {
        return Vec::new();
    }
}

impl From<&Vec<u8>> for EmptyRequest {
    fn from(raw: &Vec<u8>) -> Self {
        if raw.len() != 0 {
            // Empty requests must be empty
            return EmptyRequest::new_broken();
        }

        return EmptyRequest::new(MessageType::Unknown);
    }
}

struct SubscriptionRequest {
    pub class: MessageType,
    pub sub_id: String,
}

impl SubscriptionRequest {
    fn new(class: MessageType, sub_id: String) -> SubscriptionRequest {
        return SubscriptionRequest{
            class: class,
            sub_id: sub_id,
        }
    }

    fn new_broken() -> SubscriptionRequest {
        return SubscriptionRequest{
            class: MessageType::Invalid,
            sub_id: String::new()
        }
    }
}

impl From<&Vec<u8>> for SubscriptionRequest {
    fn from(raw: &Vec<u8>) -> Self {
        if raw.len() != 20 {
            // Size must be 20 for the subscription id (utf8 encoded guid)
            return SubscriptionRequest::new_broken();
        }
        let mut bytes = raw.into_iter();

        let subscription_id = match String::from_utf8(bytes.map(|b| *b).collect()) {
            Err(_err) => {
                eprintln!("Failed to parse into SubscriptionRequest: SubscriptionID is not utf8");
                // Rest of the bytes are invalid utf-8
                return SubscriptionRequest::new_broken();
            },
            Ok(s) => s
        };

        return SubscriptionRequest::new(MessageType::Unknown, String::from(subscription_id));
    }
}

pub struct PutMessageRequest {
    pub class: MessageType,
    pub topic_id: String,
    pub message: Vec<u8>
}

impl PutMessageRequest {
    fn new(topic_id: String, message: Vec<u8>) -> PutMessageRequest {
        return PutMessageRequest{
            class: MessageType::ProduceMessage,
            topic_id: topic_id,
            message: message,
        }
    }

    fn new_broken() -> PutMessageRequest {
        return PutMessageRequest{
            class: MessageType::Invalid,
            topic_id: String::new(),
            message: Vec::new(),
        }
    }
}

impl From<&Vec<u8>> for PutMessageRequest {
    fn from(raw: &Vec<u8>) -> Self {
        if raw.len() < 2 {
            // Minimum size is 2 bytes for topic name size
            return PutMessageRequest::new_broken();
        }
        let mut bytes = raw.iter();

        let topic_name_length = ((*bytes.next().unwrap() as u16) << 8 | (*bytes.next().unwrap() as u16)) as usize;
        if raw.len() - 2 < topic_name_length {
            eprintln!("Failed to parse into PutMessageRequest: Topic length {}, but {} bytes left", topic_name_length, raw.len()-2);
            // Rest of the bytes are the wrong length
            return PutMessageRequest::new_broken();
        }

        let (topic_name_iter, message_iter): (Vec<(usize, &u8)>, Vec<(usize, &u8)>) = bytes.enumerate().partition(|(i, _)| i < &topic_name_length);

        let topic_name = match String::from_utf8(topic_name_iter.into_iter().map(|(_, b)| *b).collect()) {
            Err(_err) => {
                eprintln!("Failed to parse into PutMessageRequest: TopicName is not utf8");
                // Rest of the bytes are invalid utf-8
                return PutMessageRequest::new_broken();
            },
            Ok(s) => s
        };

        let message: Vec<u8> = message_iter.into_iter().map(|(_, b)| *b).collect();

        return PutMessageRequest::new(topic_name, message);
    }
}

pub fn parse_as_put_message_request(raw: &Vec<u8>) -> Option<PutMessageRequest> {
    let mut request = PutMessageRequest::from(raw);
    if request.class == MessageType::Invalid {
        return None;
    }
    request.class = MessageType::ProduceMessage;
    return Some(request);
}
