use super::device::{Device,UUID};
use mem::read_struct;
use std::io::{Error, SeekFrom, Seek};
use std::fs::File;

pub trait SuperBlock {
    fn get_label(&self) -> Option<String>;
    fn get_uuid(&self) -> Option<UUID>;
    fn validate_superblock(&self) -> bool;
}

pub trait OffsetSuperBlock : SuperBlock {
    fn get_superblock_offset() -> u64;
}

pub trait FromDevice where Self: Sized, Self: OffsetSuperBlock {
    fn from_raw_device(&Device) -> Option<Box<dyn SuperBlock>>;
    fn from_raw_device_unchecked(&Device) -> Result<Self, Error>;
}

impl<T: 'static> FromDevice for T where T: OffsetSuperBlock {
    fn from_raw_device_unchecked(d: &Device) -> Result<Self, Error> {
        let mut f = File::open(d.get_path())?;
        // First, seek into the file to the superblock offset
        if let Err(err) = f.seek(SeekFrom::Start(T::get_superblock_offset())) {
            // If we can't seek in, there's no superblock and this isn't the expected file system
            return Err(err);
        }

        return read_struct(&mut f);
    }

    fn from_raw_device(d: &Device) -> Option<Box<dyn SuperBlock>> {
        if let Ok(superblock) = Self::from_raw_device_unchecked(d) {
            if superblock.validate_superblock() {
                return Some(Box::new(superblock));
            }
        }

        return None;
    }
}