use super::super::error::NetLinkError;
use io::{read_u16, write_u16, BufferReader, Endianness};
use std::io::{Read, Write};

const ALIGN_TO: u16 = 4;

pub fn read_new_attr<T: Read>(reader: &mut T) -> Result<(u16, Vec<u8>), NetLinkError> {
    let length: u16 = read_u16(reader, &Endianness::Little)?;
    let attr_type: u16 = read_u16(reader, &Endianness::Little)?;
    let padding_length: u32 = (((length + ALIGN_TO - 1) & !(ALIGN_TO - 1)) - length) as u32;

    let mut data_buffer = vec![0; length as usize - 4];
    reader.read_exact(&mut data_buffer)?;

    let mut _padding_buffer = vec![0; padding_length as usize];
    reader.read_exact(&mut _padding_buffer)?;

    return Ok((attr_type, data_buffer));
}

pub fn write_routing_attribute<T: Write>(writer: &mut T, attr_type: u16, data: &Vec<u8>) -> Result<(), NetLinkError> {
    let length = (data.len() + 4) as u16;
    let padding_length: u32 = (((length + ALIGN_TO - 1) & !(ALIGN_TO - 1)) - length) as u32;
    let mut attr = Vec::new();
    write_u16(&mut attr, length, &Endianness::Little);
    write_u16(&mut attr, attr_type, &Endianness::Little);
    attr.append(&mut data.clone());
    attr.append(&mut (0..padding_length).map(|_| 0 as u8).collect());
    writer.write_all(&mut attr)?;

    let mut read = BufferReader::new(&attr);

    println!("{:?}", read_new_attr(&mut read));

    return Ok(());
}
