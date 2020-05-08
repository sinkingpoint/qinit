use std::path::PathBuf;
use std::fs::read_link;
use std::fmt;
use io::to_absolute_from_relative;
use super::ext::{ExtSuperBlock, Ext3SuperBlock, Ext4SuperBlock};
use super::fat::{FAT12SuperBlock, FAT16SuperBlock, FAT32SuperBlock};
use super::btrfs::BtrfsSuperBlock;
use super::superblock::{SuperBlock, FromDevice};

/// Represents a Device in the /dev file system which can be probed
#[derive(Debug)]
pub struct Device {
    path: PathBuf
}

impl Device {
    /// Constructs a Device from a given UUID, returning an error string if it can't find it
    /// At the moment, uses the /dev/disk/by-uuid links to find it
    pub fn from_uuid(uuid: &String) -> Result<Device, String> {
        let link = match read_link(format!("/dev/disk/by-uuid/{}", uuid)) {
            Ok(link) => to_absolute_from_relative(&PathBuf::from("/dev/disk/by-uuid"), &link).unwrap(),
            Err(_err) => {
                return Err(format!("No disk with UUID {} found", uuid));
            }
        };

        return Ok(Device{
            path: link
        });
    }

    /// Constructs a device from a given path
    pub fn from_path(path: PathBuf) -> Result<Device, String> {
        return Ok(Device{
            path: path
        });
    }

    pub fn get_path(&self) -> PathBuf {
        return self.path.clone();
    }

    pub fn probe(&self) -> Option<ProbeResult> {
        let probes: Vec<Box<Prober>> = vec![
            Box::new(Prober{
                name: "ext4",
                version: "1.0",
                usage: "filesystem",
                probe: &Ext4SuperBlock::from_raw_device
            }),
            Box::new(Prober{
                name: "ext3",
                version: "1.0",
                usage: "filesystem",
                probe: &Ext3SuperBlock::from_raw_device
            }),
            Box::new(Prober{
                name: "ext2",
                version: "1.0",
                usage: "filesystem",
                probe: &ExtSuperBlock::from_raw_device
            }),
            Box::new(Prober{
                name: "btrfs",
                version: "",
                usage: "filesystem",
                probe: &BtrfsSuperBlock::from_raw_device
            }),
            Box::new(Prober{
                name: "vfat",
                version: "fat32",
                usage: "filesystem",
                probe: &FAT32SuperBlock::from_raw_device
            }),
            Box::new(Prober{
                name: "vfat",
                version: "fat32",
                usage: "filesystem",
                probe: &FAT16SuperBlock::from_raw_device
            }),
            Box::new(Prober{
                name: "vfat",
                version: "fat12",
                usage: "filesystem",
                probe: &FAT12SuperBlock::from_raw_device
            })
        ];
    
        for probe in probes.iter() {
            if let Some(superblock) = (probe.probe)(self) {
                return Some(ProbeResult::from(self, probe, superblock));
            }
        }
    
        return None;
    }
}

/// Represents a variable length (either 16 or 8 bytes) UUID for a device
/// Has the ability to print them nicely as a string
pub struct UUID {
    bytes: Vec<u8>
}

impl UUID {
    /// Constructs a 16 byte uuid from a 16 u8 long slice, printed as xxxx-xx-xx-xx-xxxxxx
    pub fn from_slice16(bytes: [u8;16]) -> UUID {
        return UUID{
            bytes: bytes.iter().map(|x| *x).collect()
        }
    }

    /// Constructs a 8 byte uuid from a 8 u8 long slice, printed as xxxx-xxxx
    pub fn from_slice8(bytes: [u8;8]) -> UUID {
        return UUID{
            bytes: bytes.iter().map(|x| *x).collect()
        }
    }
}

impl fmt::Display for UUID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let parts: Vec<u8>;
        if self.bytes.len() == 16 {
            parts = vec![4, 2, 2, 2, 6];
        }
        else if self.bytes.len() == 8 {
            parts = vec![4, 4];
        }
        else {
            return Ok(());
        }
        
        let mut i = 0;
        for part_index in 0..parts.len() {
            for j in 0..parts[part_index] {
                write!(f, "{:02x}", self.bytes[(i+j) as usize])?;
            }

            if part_index < parts.len() - 1 {
                write!(f, "-")?;
            }

            i += parts[part_index];
        }
        
        return Ok(());
    }
}

struct Prober {
    name: &'static str,
    version: &'static str,
    usage: &'static str,
    probe: &'static dyn Fn(&Device) -> Option<Box<dyn SuperBlock>>
}

pub struct ProbeResult {
    path: PathBuf,
    fs_name: String,
    version: String,
    usage: String,
    label: Option<String>,
    uuid: Option<UUID>,
}

impl ProbeResult {
    fn from(device: &Device, p: &Prober, superblock: Box<dyn SuperBlock>) -> ProbeResult {
        return ProbeResult {
            path: device.get_path(),
            fs_name: String::from(p.name),
            version: String::from(p.version),
            usage: String::from(p.usage),
            label: superblock.get_label(),
            uuid: superblock.get_uuid(),
        };
    }

    pub fn get_fs(&self) -> String {
        return self.fs_name.clone();
    }
}

impl fmt::Display for ProbeResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: ", self.path.to_str().unwrap())?;
        if let Some(uuid) = &self.uuid {
            write!(f, "UUID=\"{}\" ", uuid)?;
        }

        if let Some(label) = &self.label {
            write!(f, "LABEL=\"{}\" ", label)?;
        }

        if self.version != "" {
            write!(f, "VERSION=\"{}\" ", &self.version)?;
        }
        
        write!(f, "TYPE=\"{}\" ", &self.fs_name)?;
        write!(f, "USAGE=\"{}\" ", &self.usage)?;

        return Ok(());
    }
}
