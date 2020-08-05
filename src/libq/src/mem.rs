use std::io::{self, Read};

pub unsafe fn read_struct<T, R: Read>(read: &mut R) -> io::Result<T> {
    let num_bytes = ::std::mem::size_of::<T>();
    let mut s = std::mem::MaybeUninit::<T>::uninit();
    let buffer = std::slice::from_raw_parts_mut((&mut s).as_ptr() as *mut T as *mut u8, num_bytes);
    match read.read_exact(buffer) {
        Ok(()) => Ok(s.assume_init()),
        Err(e) => {
            ::std::mem::forget(s);
            Err(e)
        }
    }
}

pub unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    ::std::slice::from_raw_parts((p as *const T) as *const u8, ::std::mem::size_of::<T>())
}

#[inline]
pub fn short_from_bytes_little_endian(a: u8, b: u8) -> u16 {
    return ((b as u16) << 8) | a as u16;
}

#[inline]
pub fn int_from_bytes_little_endian(a: u8, b: u8, c: u8, d: u8) -> u32 {
    return ((d as u32) << 24) | ((c as u32) << 16) | ((b as u32) << 8) | a as u32;
}

#[inline]
pub fn long_from_bytes_little_endian(a: u8, b: u8, c: u8, d: u8, e: u8, f: u8, g: u8, h: u8) -> u64 {
    return ((h as u64) << 56)
        | ((g as u64) << 48)
        | ((f as u64) << 40)
        | ((e as u64) << 32)
        | ((d as u64) << 24)
        | ((c as u64) << 16)
        | ((b as u64) << 8)
        | a as u64;
}

pub trait ReadableFromRaw {
    unsafe fn read_from_raw<R: Read>(read: &mut R) -> io::Result<Self> where Self: Sized;
}

impl<T> ReadableFromRaw for T where Self: Sized {
    unsafe fn read_from_raw<R: Read>(read: &mut R) -> io::Result<Self> {
        return read_struct(read);
    }
}