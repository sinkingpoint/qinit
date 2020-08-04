use super::device::UUID;
use super::superblock::{OffsetSuperBlock, SuperBlock};
use strings::cstr_to_string;

/// BtrfsSuperBlock is a packed struct of the on disk structure of a
/// superblock of a Btrfs file system
#[repr(C, packed)]
pub struct BtrfsSuperBlock {
    checksum: [u8; 0x20],
    uuid: [u8; 0x10],
    physical_address: u64,
    flags: u64,
    magic: [u8; 0x8],
    generation: u64,
    root_tree_logical: u64,
    chunk_tree_logical: u64,
    log_tree_logical: u64,
    log_root_transid: u64,
    total_bytes: u64,
    bytes_used: u64,
    root_dir_objectid: u64,
    num_devices: u64,
    sectorsize: u32,
    nodesize: u32,
    leafsize: u32,
    stripesize: u32,
    sys_chunk_array_size: u32,
    compat_flags: u64,
    compat_ro_flags: u64,
    incompat_flags: u64,
    csum_type: u16,
    root_level: u8,
    chunk_root_level: u8,
    log_root_level: u8,
    dev_items: [u16; 0x32],
    label: [u8; 0x100],
    cache_generation: u64,
    uuid_tree_generation: u64,
}

impl OffsetSuperBlock for BtrfsSuperBlock {
    fn get_superblock_offset() -> u64 {
        // The first Btrfs superblock sits at 64k into the file system == 0x10000 bytes
        return 0x10000;
    }
}

impl SuperBlock for BtrfsSuperBlock {
    fn get_label(&self) -> Option<String> {
        if let Ok(cstr) = cstr_to_string(&self.label) {
            return Some(cstr);
        }
        return None;
    }

    fn get_uuid(&self) -> Option<UUID> {
        return Some(UUID::from_slice16(self.uuid));
    }

    /// Returns true if the superblock's magic number is correct. Btrfs superblocks have "_BHRfS_M" as the magic number
    /// [95, 66, 72, 82, 102, 83, 95, 77] == ['B', 'H', 'R', 'f', 'S', '_', 'M'] in ASCII/UTF-8
    fn validate_superblock(&self) -> bool {
        return self.magic == [95, 66, 72, 82, 102, 83, 95, 77];
    }
}
