use std::convert::TryFrom;
use std::io::{self, Read, Write};
use libq::io::{read_u32, write_u32, Endianness, Writable};

use super::types::UUID;

#[derive(Debug)]
pub enum MessageType {
    CreateTopic,
    DeleteTopic,
    Subscribe,
    Unsubscribe,
    ProduceMessage,
    ConsumeMessage,
    GetTopics,
    GetNumSubscribers,
}

impl TryFrom<u32> for MessageType {
    type Error = FreudianAPIError;
    fn try_from(u: u32) -> Result<Self, Self::Error> {
        return match u {
            0 => Ok(MessageType::CreateTopic),
            1 => Ok(MessageType::DeleteTopic),
            2 => Ok(MessageType::Subscribe),
            3 => Ok(MessageType::Unsubscribe),
            4 => Ok(MessageType::ProduceMessage),
            5 => Ok(MessageType::ConsumeMessage),
            6 => Ok(MessageType::GetTopics),
            7 => Ok(MessageType::GetNumSubscribers),
            _ => Err(FreudianAPIError::MalformedRequest)
        }
    }
}

impl MessageType {
    fn as_u32(&self) -> u32 {
        return match self {
            MessageType::CreateTopic => 0,
            MessageType::DeleteTopic => 1,
            MessageType::Subscribe => 2,
            MessageType::Unsubscribe => 3,
            MessageType::ProduceMessage => 4,
            MessageType::ConsumeMessage => 5,
            MessageType::GetTopics => 6,
            MessageType::GetNumSubscribers => 7
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FreudianAPIResponseType {
    Ok,
    NothingHappened,
    No,
    DoesntExist,
    MalformedRequest,
    ServerError
}

impl TryFrom<u32> for FreudianAPIResponseType {
    type Error = FreudianAPIError;
    fn try_from(u: u32) -> Result<Self, Self::Error> {
        return match u {
            0 => Ok(FreudianAPIResponseType::Ok),
            1 => Ok(FreudianAPIResponseType::NothingHappened),
            2 => Ok(FreudianAPIResponseType::No),
            3 => Ok(FreudianAPIResponseType::DoesntExist),
            4 => Ok(FreudianAPIResponseType::MalformedRequest),
            5 => Ok(FreudianAPIResponseType::ServerError),
            _ => Err(FreudianAPIError::MalformedRequest)
        }
    }
}

impl Into<u32> for FreudianAPIResponseType {
    fn into(self) -> u32 {
        match self {
            FreudianAPIResponseType::Ok => 0,
            FreudianAPIResponseType::NothingHappened => 1,
            FreudianAPIResponseType::No => 2,
            FreudianAPIResponseType::DoesntExist => 3,
            FreudianAPIResponseType::MalformedRequest => 4,
            FreudianAPIResponseType::ServerError => 5
        }
    }
}

pub enum FreudianAPIError {
    IOError(io::Error),
    MalformedRequest
}

impl From<io::Error> for FreudianAPIError {
    fn from(err: io::Error) -> FreudianAPIError {
        return FreudianAPIError::IOError(err);
    }
}

pub struct FreudianAPIResponse {
    pub response_type: FreudianAPIResponseType,
    pub message: Vec<u8>
}

impl FreudianAPIResponse {
    pub fn empty(response_type: FreudianAPIResponseType) -> FreudianAPIResponse{
        return FreudianAPIResponse {
            response_type: response_type,
            message: Vec::new()
        }
    }

    pub fn with_message(response_type: FreudianAPIResponseType, message: Vec<u8>) -> FreudianAPIResponse{
        return FreudianAPIResponse {
            response_type: response_type,
            message: message
        }
    }

    pub fn read<T: Read>(reader: &mut T) -> Result<FreudianAPIResponse, io::Error> {
        let endianness = &Endianness::Little;
        let response_type = match FreudianAPIResponseType::try_from(read_u32(reader, endianness)?) {
            Ok(resp) => resp,
            Err(_) => FreudianAPIResponseType::MalformedRequest
        };

        let message_length = read_u32(reader, endianness)? as usize;
        let mut message_buffer = vec![0; message_length];
        reader.read_exact(&mut message_buffer)?;

        return Ok(FreudianAPIResponse {
            response_type: response_type,
            message: message_buffer
        })
    }
}

impl Writable for FreudianAPIResponse {
    fn write<T: Write>(&self, writer: &mut T) -> Result<(), io::Error> {
        let endianness = &Endianness::Little;
        write_u32(writer, self.response_type.into(), endianness)?;
        write_u32(writer, self.message.len() as u32, endianness)?;

        let mut buf = self.message.clone();
        return writer.write_all(&mut buf);
    }
}

impl From<FreudianAPIResponseType> for FreudianAPIResponse {
    fn from(code: FreudianAPIResponseType) -> FreudianAPIResponse {
        return FreudianAPIResponse::empty(code);
    }
}

#[derive(Debug)]
pub struct FreudianRequestHeader {
    pub message_type: MessageType,
    pub message_length: u32
}

impl FreudianRequestHeader {
    pub fn new(message_type: MessageType, body_length: u32) -> FreudianRequestHeader {
        return FreudianRequestHeader {
            message_type: message_type,
            message_length: body_length + 8 // 8 bytes is the size of this header
        }
    }

    pub fn size(&self) -> u32 {
        return 8;
    }
    
    pub fn read<T: Read>(reader: &mut T) -> Result<FreudianRequestHeader, FreudianAPIError> {
        let endian = &Endianness::Little;
        return Ok(FreudianRequestHeader {
            message_type: MessageType::try_from(read_u32(reader, endian)?)?,
            message_length: read_u32(reader, endian)?
        });
    }
}

impl Writable for FreudianRequestHeader {
    fn write<T: Write>(&self, writer: &mut T) -> Result<(), io::Error> {
        let endianness = &Endianness::Little;
        write_u32(writer, self.message_type.as_u32(), endianness)?;
        return write_u32(writer, self.message_length, endianness);
    }
}

#[derive(Debug)]
pub struct FreudianTopicRequest {
    topic_name_length: u32,
    pub topic_name: Vec<u8>
}

impl FreudianTopicRequest {
    pub fn new(topic_name: String) -> FreudianTopicRequest {
        return FreudianTopicRequest {
            topic_name_length: topic_name.len() as u32,
            topic_name: topic_name.bytes().collect()
        }
    }

    pub fn size(&self) -> u32 {
        return self.topic_name.len() as u32 + 4;
    }

    pub fn read<T: Read>(reader: &mut T) -> Result<FreudianTopicRequest, FreudianAPIError> {
        let endian = &Endianness::Little;

        let length = read_u32(reader, endian)?;
        let mut name_buffer = vec![0; length as usize];
        reader.read_exact(&mut name_buffer)?;

        return Ok(FreudianTopicRequest {
            topic_name_length: length,
            topic_name: name_buffer
        });
    }
}

impl Writable for FreudianTopicRequest {
    fn write<T: Write>(&self, writer: &mut T) -> Result<(), io::Error> {
        let endian = &Endianness::Little;

        write_u32(writer, self.topic_name_length, endian)?;
        let mut data = self.topic_name.clone();
        return writer.write_all(&mut data);
    }
}

pub struct FreudianProduceMessageRequest {
    pub topic_request: FreudianTopicRequest,
    pub message: Vec<u8>
}

impl FreudianProduceMessageRequest {
    pub fn new(topic_name: String, message: Vec<u8>) -> FreudianProduceMessageRequest {
        return FreudianProduceMessageRequest {
            topic_request: FreudianTopicRequest::new(topic_name),
            message: message
        }
    }

    pub fn size(&self) -> u32 {
        return (4 + self.topic_request.topic_name.len() + self.message.len()) as u32;
    }

    pub fn read<T: Read>(reader: &mut T, body_length: u32) -> Result<FreudianProduceMessageRequest, FreudianAPIError> {
        let topic_request = FreudianTopicRequest::read(reader)?;
        if topic_request.topic_name_length >= body_length {
            // If the topic size is bigger than the header reported for the whole message, bail
            return Err(FreudianAPIError::MalformedRequest);
        }

        let mut message_buffer = vec![0;(body_length - topic_request.size()) as usize];
        reader.read_exact(&mut message_buffer)?;

        return Ok(FreudianProduceMessageRequest {
            topic_request: topic_request,
            message: message_buffer
        });
    }
}

impl Writable for FreudianProduceMessageRequest {
    fn write<T: Write>(&self, writer: &mut T) -> Result<(), io::Error> {
        self.topic_request.write(writer)?;
        let mut buffer = self.message.clone();
        return writer.write_all(&mut buffer);
    }
}


#[derive(Clone)]
pub struct FreudianSubscriptionRequest {
    pub subscription_id: [u8;16]
}

impl FreudianSubscriptionRequest {
    pub fn new(sub_id: [u8;16]) -> FreudianSubscriptionRequest {
        return FreudianSubscriptionRequest {
            subscription_id: sub_id
        }
    }

    pub fn read<T: Read>(reader: &mut T) -> Result<FreudianSubscriptionRequest, FreudianAPIError> {
        let mut sub_buffer = [0;16];
        reader.read_exact(&mut sub_buffer)?;
        return Ok(FreudianSubscriptionRequest {
            subscription_id: sub_buffer
        });
    }

    pub fn size(&self) -> u32 {
        return 16;
    }
}

impl From<UUID> for FreudianSubscriptionRequest {
    fn from(uuid: UUID) -> FreudianSubscriptionRequest {
        return FreudianSubscriptionRequest::new(uuid.uuid);
    }
}

impl Writable for FreudianSubscriptionRequest {
    fn write<T: Write>(&self, writer: &mut T) -> Result<(), io::Error> {
        let mut data = self.subscription_id.clone();
        return writer.write_all(&mut data);
    }
}