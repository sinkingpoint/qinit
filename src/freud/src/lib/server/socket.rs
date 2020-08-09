use std::path::{Path};
use std::os::unix::net::{UnixListener, UnixStream};
use std::os::unix::io::AsRawFd;
use std::io::{self, Write};
use std::fs::remove_file;
use std::sync::{Arc, Mutex};
use std::thread::park_timeout;
use std::time::Duration;
use std::mem::drop;

use nix::sys::socket::sockopt::PeerCredentials;
use nix::sys::socket::getsockopt;

use libq::logger;
use libq::io::{read_u32, Endianness};

use super::api::{FreudianRequestHeader, MessageType, FreudianTopicRequest, FreudianSubscriptionRequest, FreudianProduceMessageRequest, FreudianAPIResponseType, FreudianAPIResponse, FreudianAPIError};
use super::types::{FreudianResponse, FreudianError};
use super::bus::FreudianBus;

pub enum FreudianSocketError {
    ProcessAlreadyRunning,
    IOError(io::Error)
}

impl From<io::Error> for FreudianSocketError {
    fn from(err: io::Error) -> FreudianSocketError {
        return FreudianSocketError::IOError(err);
    }
}

pub struct FreudianSocket {
    socket: UnixListener
}

fn process_request(socket: &mut UnixStream, bus_lock: &mut Arc<Mutex<FreudianBus>>, header: FreudianRequestHeader) -> Result<FreudianAPIResponse, FreudianAPIError> {
    let mut bus = bus_lock.lock().unwrap();
    let response = match header.message_type {
        MessageType::CreateTopic => bus.create_topic(FreudianTopicRequest::read(socket)?),
        MessageType::DeleteTopic => bus.delete_topic(FreudianTopicRequest::read(socket)?),
        MessageType::Subscribe => bus.subscribe(FreudianTopicRequest::read(socket)?),
        MessageType::Unsubscribe => bus.unsubscribe(FreudianSubscriptionRequest::read(socket)?),
        MessageType::ProduceMessage => bus.produce_message(FreudianProduceMessageRequest::read(socket, header.message_length)?),
        MessageType::ConsumeMessage => {
            let sub_request = FreudianSubscriptionRequest::read(socket)?;
            let max_wait_time = read_u32(socket, &Endianness::Little)?;
            let result: Result<FreudianResponse, FreudianError> = match bus.consume_message(sub_request.clone()) {
                Ok(FreudianResponse::Message(msg)) => Ok(FreudianResponse::Message(msg)),
                Ok(_) => Err(FreudianError::InvalidResponse),
                Err(FreudianError::SubscriptionDoesntExist) => Err(FreudianError::SubscriptionDoesntExist),
                Err(FreudianError::NoNewMessages) => {
                    if max_wait_time == 0 {
                        Result::<FreudianResponse, FreudianError>::Err(FreudianError::NoNewMessages)
                    }
                    else {
                        // Mark this thread as waiting on the given topic
                        bus.mark_thread_waiting(sub_request.clone()).expect("Failed to mark thread as waiting");

                        // Manually release the lock before we sleep the thread
                        drop(bus);

                        park_timeout(Duration::new(max_wait_time as u64, 0));

                        bus = bus_lock.lock().unwrap();

                        match bus.consume_message(sub_request.clone()) {
                            Ok(FreudianResponse::Message(msg)) => Ok(FreudianResponse::Message(msg)),
                            Ok(_) => Err(FreudianError::InvalidResponse),
                            Err(FreudianError::SubscriptionDoesntExist) => Err(FreudianError::SubscriptionDoesntExist),
                            Err(FreudianError::NoNewMessages) => Err(FreudianError::NoNewMessages),
                            Err(e) => Err(e)
                        }
                    }
                },
                Err(_) => Err(FreudianError::InvalidResponse)
            };

            result
        }
    };

    return match response {
        Ok(FreudianResponse::Empty) => Ok(FreudianAPIResponse::empty(FreudianAPIResponseType::Ok)),
        Ok(FreudianResponse::Message(msg)) => Ok(FreudianAPIResponse::with_message(FreudianAPIResponseType::Ok, msg)),
        Ok(FreudianResponse::Subscription(sub)) => Ok(FreudianAPIResponse::with_message(FreudianAPIResponseType::Ok, sub.uuid.to_vec())),
        Err(FreudianError::InvalidResponse) => Ok(FreudianAPIResponse::empty(FreudianAPIResponseType::ServerError)),
        Err(FreudianError::InvalidString(_)) => Ok(FreudianAPIResponse::empty(FreudianAPIResponseType::MalformedRequest)),
        Err(FreudianError::NoNewMessages) | Err(FreudianError::NoSubscribers) | Err(FreudianError::TopicAlreadyExists) => Ok(FreudianAPIResponse::empty(FreudianAPIResponseType::NothingHappened)),
        Err(FreudianError::SubscriptionDoesntExist) | Err(FreudianError::TopicDoesntExist) =>  Ok(FreudianAPIResponse::empty(FreudianAPIResponseType::DoesntExist)),
    }
}

fn handle_client(mut socket: UnixStream, mut bus_lock: Arc<Mutex<FreudianBus>>) {
    let conn_fd = socket.as_raw_fd();
    let creds = getsockopt(conn_fd, PeerCredentials);

    // TODO: Authentication
    loop {
        let header = match FreudianRequestHeader::read(&mut socket) {
            Ok(h) => h,
            Err(FreudianAPIError::IOError(_err)) => {
                // We got an IO Error, kill the connection
                break;
            },
            Err(_err) => {
                // send error
                FreudianAPIResponse::empty(FreudianAPIResponseType::MalformedRequest).write(&mut socket).expect("Failed to write response");
                continue;
            }
        };

        match process_request(&mut socket, &mut bus_lock, header) {
            Ok(resp) => resp.write(&mut socket),
            Err(FreudianAPIError::MalformedRequest) => FreudianAPIResponse::empty(FreudianAPIResponseType::MalformedRequest).write(&mut socket),
            Err(FreudianAPIError::IOError(_err)) => {
                // We got an IO Error, kill the connection
                break;
            }
        }.expect("Failed to write response");

        socket.flush().expect("Failed to flush socket");
    }
}

impl FreudianSocket {
    pub fn new(socketfile: &Path) -> Result<FreudianSocket, FreudianSocketError> {
        if socketfile.exists() {
            // If the socket exists, try connect to it. If we can, then there must be a process already running
            match UnixStream::connect(socketfile) {
                Ok(_) => {
                    return Err(FreudianSocketError::ProcessAlreadyRunning);
                },
                Err(_) => {
                    remove_file(socketfile)?;
                }
            }
        }

        return Ok(FreudianSocket {
            socket: UnixListener::bind(socketfile)?
        });
    }

    pub fn listen_and_serve(&self) {
        let logger = logger::with_name_as_json("freudian;socket");
        let bus = Arc::new(Mutex::new(FreudianBus::new()));

        for stream in self.socket.incoming() {
            match stream {
                Ok(stream) => {
                    let bus_ref = Arc::clone(&bus);
                    std::thread::spawn(move || {
                        handle_client(stream, bus_ref);
                    })
                },
                Err(_e) => {
                    break;
                }
            };
        }
    }
}
