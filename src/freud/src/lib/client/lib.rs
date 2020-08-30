extern crate breuer;
extern crate libq;

pub use {breuer::FreudianAPIResponseType as Status, breuer::FreudianAPIResponse as MessageResponse, breuer::UUID};

use libq::io::{Writable, write_u32, Endianness};
use breuer::{FreudianRequestHeader, FreudianTopicRequest, FreudianSubscriptionRequest, FreudianProduceMessageRequest, MessageType, FreudianAPIResponse};
use std::os::unix::net::UnixStream;
use std::io::{self};
use std::path::Path;

/// FreudianClient represents a client interface to Freudian, the message bus daemon of qinit
pub struct FreudianClient {
    /// The underlying socket connection to Freudian
    socket: UnixStream
}

impl FreudianClient {
    /// Constructs a new FreudianClient instance, connected to the socket at the given path
    /// returning the client, or an error if we couldn't connect to the socket for some reason
    pub fn new(socketfile: &Path) -> Result<FreudianClient, io::Error> {
        return Ok(FreudianClient {
            socket: UnixStream::connect(socketfile)?
        });
    }

    fn send_to_socket<T: Writable>(&mut self, header: FreudianRequestHeader, body: T) -> Result<(), io::Error> {
        header.write(&mut self.socket)?;
        return body.write(&mut self.socket);
    }

    fn read_response_from_socket(&mut self) -> Result<FreudianAPIResponse, io::Error> {
        return FreudianAPIResponse::read(&mut self.socket);
    }

    fn send_topic_request(&mut self, request: MessageType, topic_name: &str) -> Result<(), io::Error> {
        let topic_request = FreudianTopicRequest::new(topic_name.to_owned());
        let header = FreudianRequestHeader::new(request, topic_request.size());
        return self.send_to_socket(header, topic_request);
    }

    fn send_subscription_request(&mut self, request: MessageType, uuid: UUID) -> Result<(), io::Error> {
        let sub_request = FreudianSubscriptionRequest::new(uuid.uuid);
        let header = FreudianRequestHeader::new(request, sub_request.size());
        return self.send_to_socket(header, sub_request);
    }

    fn send_create_message_request(&mut self, request: MessageType, topic_name: &str, message: Vec<u8>) -> Result<(), io::Error> {
        let produce_request = FreudianProduceMessageRequest::new(topic_name.to_owned(), message);
        let header = FreudianRequestHeader::new(request, produce_request.size());
        return self.send_to_socket(header, produce_request);
    }

    /// Creates a topic of the given name, returning the Status responded by Freudian
    pub fn create_topic(&mut self, topic_name: &str) -> Result<Status, io::Error> {
        self.send_topic_request(MessageType::CreateTopic, topic_name)?;
        return match self.read_response_from_socket() {
            Ok(resp) => Ok(resp.response_type),
            Err(e) => Err(e)
        };
    }

    /// Deletes the topic with the given name
    pub fn delete_topic(&mut self, topic_name: &str) -> Result<Status, io::Error> {
        self.send_topic_request(MessageType::DeleteTopic, topic_name)?;
        return match self.read_response_from_socket() {
            Ok(resp) => Ok(resp.response_type),
            Err(e) => Err(e)
        };
    }

    /// Creates a subscription to the given topic, returning the raw API response from Freudian
    /// If the response status is "Ok", the body should contain a 16 byte subscription ID which 
    /// can be used in subsequent message requests
    pub fn subscribe(&mut self, topic_name: &str) -> Result<FreudianAPIResponse, io::Error> {
        self.send_topic_request(MessageType::Subscribe, topic_name)?;
        return self.read_response_from_socket();
    }

    /// Deletes the subscription with the given subscription ID
    pub fn unsubscribe(&mut self, sub_id: UUID) -> Result<Status, io::Error> {
        self.send_subscription_request(MessageType::Unsubscribe, sub_id)?;
        return match self.read_response_from_socket() {
            Ok(resp) => Ok(resp.response_type),
            Err(e) => Err(e)
        };
    }

    /// Produces a message into the given topic
    pub fn produce_message(&mut self, topic_name: &str, message: Vec<u8>) -> Result<Status, io::Error> {
        self.send_create_message_request(MessageType::ProduceMessage, topic_name, message)?;
        return match self.read_response_from_socket() {
            Ok(resp) => Ok(resp.response_type),
            Err(e) => Err(e)
        };
    }

    /// Consumes a message with the given sub_id, blocking for a maximum of max_wait_secs
    /// (which may be 0, for no blocking)
    pub fn consume_message(&mut self, sub_id: UUID, max_wait_secs: u32) -> Result<FreudianAPIResponse, io::Error> {
        self.send_subscription_request(MessageType::ConsumeMessage, sub_id)?;
        write_u32(&mut self.socket, max_wait_secs, &Endianness::Little)?;
        return self.read_response_from_socket();
    }

    /// Sends a GetTopic Request, returning the raw API response from Freudian
    /// The response is of the format [[title_len, title];array_len] if the status is Ok
    pub fn get_topics(&mut self) -> Result<FreudianAPIResponse, io::Error> {
        let header = FreudianRequestHeader::new(MessageType::GetTopics, 0);
        self.send_to_socket(header, Vec::new())?;
        return self.read_response_from_socket();
    }

    /// Gets the number of subscribers of the given topic. The response is a single, little endian u32
    /// if the status is Ok
    pub fn get_num_subscibers(&mut self, topic_name: &str) -> Result<FreudianAPIResponse, io::Error> {
        self.send_topic_request(MessageType::GetNumSubscribers, topic_name)?;
        return self.read_response_from_socket();
    }
}