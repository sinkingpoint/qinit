use io::{Endianness, read_u16};
use std::io::Read;
use super::super::error::NetLinkError;

pub fn read_new_attr<T: Read>(reader: &mut T) -> Result<(u16, Vec<u8>), NetLinkError> {
    let length: u16 = read_u16(reader, &Endianness::Little)?;
    let attr_type: u16 = read_u16(reader, &Endianness::Little)?;
    const ALIGN_TO: u16 = 4;
    let padding_length: u32 = (((length + ALIGN_TO - 1) & !(ALIGN_TO - 1)) - length) as u32;

    let mut data_buffer = vec![0; length as usize - 4];
    reader.read_exact(&mut data_buffer)?;

    let mut _padding_buffer = vec![0; padding_length as usize];
    reader.read_exact(&mut _padding_buffer)?;

    return Ok((attr_type, data_buffer));
}
