use strings::cstr_to_string;
use super::superblock::{OffsetSuperBlock, SuperBlock};
use super::device::UUID;

#[repr(C, packed)]
struct FatSuperBlock {
    bootstrap_jump: [u8;3],
    oem_name: [u8;8],
    bytes_per_sector: u16,
    sectors_per_cluster: u8,
    reserved_sectors: u16,
    copies: u8,
    n_root_dir_entries: u16,
    n_sectors: u16,
    media_type: u8,
    sectors_per_table: u16,
    sectors_per_track: u16,
    n_heads: u16,
    hiden_sectors: u16,
}

#[repr(C, packed)]
pub struct FAT12SuperBlock {
    supersuper: FatSuperBlock,
    bootstrap: [u8;480],
    signature: u16
}

impl OffsetSuperBlock for FAT12SuperBlock{
    fn get_superblock_offset() -> u64 {
        return 0;
    }
}

impl SuperBlock for FAT12SuperBlock {
    fn get_label(&self) -> Option<String> {
        return None;
    }

    fn get_uuid(&self) -> Option<UUID> {
        return None;
    }

    fn validate_superblock(&self) -> bool {
        return self.signature == 0xaa55;
    }
}

#[repr(C, packed)]
pub struct FAT16SuperBlock {
    supersuper: FatSuperBlock,
    hidden_sectors_ext: u16,
    n_sectors: u32,
    logical_drive_number: u8,
    reserved: u8,
    extended_signature: u8,
    serial: u32,
    label: [u8;10],
    fstype: [u8;8],
    bootstrap: [u8;449],
    signature: u16
}

impl OffsetSuperBlock for FAT16SuperBlock{
    fn get_superblock_offset() -> u64 {
        return 0;
    }
}

impl SuperBlock for FAT16SuperBlock {
    fn get_label(&self) -> Option<String> {
        // FAT default is for the NULL label to be "NO NAME   " == (this slice in ascii)
        if self.label == [78, 79, 32, 78, 65, 77, 69, 32, 32, 32] {
            return None;
        }

        if let Ok(cstr) = cstr_to_string(&self.label) {
            return Some(cstr);
        }
        return None;
    }

    fn get_uuid(&self) -> Option<UUID> {
        return None;
    }

    fn validate_superblock(&self) -> bool {
        return self.signature == 0xaa55 && (self.extended_signature == 0x29 || self.extended_signature == 0x28);
    }
}

#[repr(C, packed)]
pub struct FAT32SuperBlock {
    supersuper: FatSuperBlock,
    hidden_sectors_ext: u16,
    n_sectors: u32,
    sectors_per_table: u32,
    mirror_flags: u16,
    fsversion: u16,
    first_root_cluster: u32,
    fs_info_sector: u16,
    backup_boot_sector: u16,
    reserved: [u8;12],
    drive_number: u8,
    reserved2: u8,
    extended_signature: u8,
    serial_number: u32,
    label: [u8;11],
    fstype: [u8; 8]
}

impl OffsetSuperBlock for FAT32SuperBlock{
    fn get_superblock_offset() -> u64 {
        return 0;
    }
}

impl SuperBlock for FAT32SuperBlock {
    fn get_label(&self) -> Option<String> {
        // FAT default is for the NULL label to be "NO NAME   " == (this slice in ascii)
        if self.label == [78, 79, 32, 78, 65, 77, 69, 32, 32, 32, 32] {
            return None;
        }
        if let Ok(cstr) = cstr_to_string(&self.label) {
            return Some(cstr);
        }
        return None;
    }

    fn get_uuid(&self) -> Option<UUID> {
        return None;
    }

    fn validate_superblock(&self) -> bool {
        return self.extended_signature == 0x29 && self.fstype == [70, 65, 84, 51, 50, 32, 32, 32];
    }
}