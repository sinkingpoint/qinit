use super::api::MessageType;
use num_enum::TryFromPrimitiveError;
use std::array::TryFromSliceError;
use std::ffi::FromBytesWithNulError;
use std::io;
use std::str::Utf8Error;

#[derive(Debug)]
pub enum NetLinkError {
    IOError(io::Error),
    NixError(nix::Error),
    IncorrectBufferSize,
    InvalidString,
    InvalidMessageType,
    InvalidEnumPrimitive(u64),
}

impl From<io::Error> for NetLinkError {
    fn from(e: io::Error) -> NetLinkError {
        return NetLinkError::IOError(e);
    }
}

impl From<nix::Error> for NetLinkError {
    fn from(e: nix::Error) -> NetLinkError {
        return NetLinkError::NixError(e);
    }
}

impl From<Utf8Error> for NetLinkError {
    fn from(_e: Utf8Error) -> NetLinkError {
        return NetLinkError::InvalidString;
    }
}

impl From<FromBytesWithNulError> for NetLinkError {
    fn from(_e: FromBytesWithNulError) -> NetLinkError {
        return NetLinkError::InvalidString;
    }
}

impl From<TryFromSliceError> for NetLinkError {
    fn from(_e: TryFromSliceError) -> NetLinkError {
        return NetLinkError::IncorrectBufferSize;
    }
}

impl From<TryFromPrimitiveError<MessageType>> for NetLinkError {
    fn from(e: TryFromPrimitiveError<MessageType>) -> NetLinkError {
        println!("{:?}", e);
        return NetLinkError::InvalidMessageType;
    }
}
