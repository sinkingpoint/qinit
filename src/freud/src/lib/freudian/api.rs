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
            Err(err) => {
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

struct EmptyRequest {
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