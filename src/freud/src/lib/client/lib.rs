extern crate libfreudian;

use std::cmp::min;
use std::io::{self, Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

use libfreudian::api::{MessageType, OneValueRequest, PutMessageRequest};

pub use libfreudian::api::ResponseType;

const MAX_MESSAGE_LENGTH: usize = 1 << 16 - 1;

pub enum FreudianClientError {
    EmptyResponse,
    ResponseTooSmall,
    ExtraData,
    ServerResponse(ResponseType),
    BrokenSocket,
}

fn make_request(class: MessageType, msg: Vec<u8>) -> Vec<u8> {
    let message_length = min(msg.len(), MAX_MESSAGE_LENGTH);
    let mut message = vec![class.into(), ((message_length & 0xFF00) >> 8) as u8, (message_length & 0xFF) as u8];
    for byte in msg.iter() {
        message.push(*byte);
    }

    return message;
}

fn decode_status_output(data: Vec<u8>) -> Result<ResponseType, FreudianClientError> {
    if data.len() == 0 {
        return Err(FreudianClientError::EmptyResponse);
    }

    let mut iter = data.iter();
    let result = *iter.next().unwrap();

    let message_size = (*iter.next().unwrap_or(&0) as u16) << 8 | (*iter.next().unwrap_or(&0) as u16);
    if message_size == 0 {
        return Ok(ResponseType::from(result));
    }

    return Err(FreudianClientError::ExtraData); // We received a non empty body, while we assume an empty one
}

fn decode_message_output(data: Vec<u8>) -> Result<(ResponseType, String), FreudianClientError> {
    if data.len() == 0 {
        return Err(FreudianClientError::EmptyResponse);
    }
    let mut iter = data.iter();
    let result = *iter.next().unwrap();
    let response = ResponseType::from(result);

    let message_size = (*iter.next().unwrap_or(&0) as u16) << 8 | (*iter.next().unwrap_or(&0) as u16);
    if message_size == 0 || data.len() < (3 + message_size) as usize {
        if response != ResponseType::Ok {
            return Err(FreudianClientError::ServerResponse(response));
        }
        return Err(FreudianClientError::ResponseTooSmall);
    }

    return match String::from_utf8(iter.map(|x| *x).collect()) {
        Ok(s) => Ok((response, s)),
        Err(_) => Err(FreudianClientError::ServerResponse(response)),
    };
}

pub struct FreudianClient {
    connection: UnixStream,
}

impl FreudianClient {
    pub fn new(socket: PathBuf) -> Result<FreudianClient, io::Error> {
        return match UnixStream::connect(socket) {
            Ok(connection) => Ok(FreudianClient { connection: connection }),
            Err(err) => Err(err),
        };
    }

    pub fn create_topic(&mut self, topic_name: &str) -> Result<ResponseType, FreudianClientError> {
        let request = make_request(
            MessageType::CreateTopic,
            OneValueRequest::new(MessageType::CreateTopic, topic_name.to_string()).into_bytes(),
        );
        return match self.send_to_socket(request) {
            Ok(response) => decode_status_output(response),
            Err(_) => Err(FreudianClientError::BrokenSocket),
        };
    }

    pub fn delete_topic(&mut self, topic_name: &str) -> Result<ResponseType, FreudianClientError> {
        let request = make_request(
            MessageType::DeleteTopic,
            OneValueRequest::new(MessageType::DeleteTopic, topic_name.to_string()).into_bytes(),
        );
        return match self.send_to_socket(request) {
            Ok(response) => decode_status_output(response),
            Err(_) => Err(FreudianClientError::BrokenSocket),
        };
    }

    pub fn create_subscription(&mut self, topic_name: &str) -> Result<(ResponseType, String), FreudianClientError> {
        let request = make_request(
            MessageType::Subscribe,
            OneValueRequest::new(MessageType::Subscribe, topic_name.to_string()).into_bytes(),
        );
        return match self.send_to_socket(request) {
            Ok(response) => decode_message_output(response),
            Err(_) => Err(FreudianClientError::BrokenSocket),
        };
    }

    pub fn read_message(&mut self, sub_id: &str) -> Result<(ResponseType, String), FreudianClientError> {
        let request = make_request(
            MessageType::GetMessage,
            OneValueRequest::new(MessageType::GetMessage, sub_id.to_string()).into_bytes(),
        );
        return match self.send_to_socket(request) {
            Ok(response) => decode_message_output(response),
            Err(_) => Err(FreudianClientError::BrokenSocket),
        };
    }

    pub fn send_message(&mut self, topic_id: &str, message: &str) -> Result<ResponseType, FreudianClientError> {
        let request = make_request(
            MessageType::ProduceMessage,
            PutMessageRequest::new(topic_id.to_string(), message.bytes().collect()).into_bytes(),
        );
        return match self.send_to_socket(request) {
            Ok(response) => decode_status_output(response),
            Err(_) => Err(FreudianClientError::BrokenSocket),
        };
    }

    fn send_to_socket(&mut self, data: Vec<u8>) -> Result<Vec<u8>, io::Error> {
        self.connection.write_all(&data[..])?;

        let mut buffer: Vec<u8> = vec![0, 0, 0];
        self.connection.read_exact(&mut buffer)?;
        let message_size = ((buffer[1] as u16) << 8 | buffer[2] as u16) as usize;
        let mut message_buffer = vec![0; message_size];
        self.connection.read_exact(&mut message_buffer)?;

        buffer.append(&mut message_buffer);

        return Ok(buffer);
    }
}
