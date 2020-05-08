use strings::cstr_to_string;
use super::superblock::{OffsetSuperBlock, SuperBlock};
use super::device::UUID;

const EXT_SB_OFF: u64 = 0x400;

const EXT_MAGIC: u16 = 0xEF53;

/// for s_feature_compat
const EXT3_FEATURE_COMPAT_HAS_JOURNAL: u32 = 0x0004;

/* for s_feature_ro_compat */
const EXT2_FEATURE_RO_COMPAT_SPARSE_SUPER: u32 = 0x0001;
const EXT2_FEATURE_RO_COMPAT_LARGE_FILE: u32   = 0x0002;
const EXT2_FEATURE_RO_COMPAT_BTREE_DIR: u32    = 0x0004;

/* for s_feature_incompat */
const EXT2_FEATURE_INCOMPAT_FILETYPE: u32      = 0x0002;
const EXT3_FEATURE_INCOMPAT_RECOVER: u32       = 0x0004;
const EXT3_FEATURE_INCOMPAT_JOURNAL_DEV: u32   = 0x0008;
const EXT2_FEATURE_INCOMPAT_META_BG: u32       = 0x0010;

const EXT2_FEATURE_RO_COMPAT_SUPP: u32          = (EXT2_FEATURE_RO_COMPAT_SPARSE_SUPER | EXT2_FEATURE_RO_COMPAT_LARGE_FILE | EXT2_FEATURE_RO_COMPAT_BTREE_DIR);
const EXT2_FEATURE_INCOMPAT_SUPP: u32           = (EXT2_FEATURE_INCOMPAT_FILETYPE | EXT2_FEATURE_INCOMPAT_META_BG);
const EXT2_FEATURE_INCOMPAT_UNSUPPORTED: u32	= !EXT2_FEATURE_INCOMPAT_SUPP;
const EXT2_FEATURE_RO_COMPAT_UNSUPPORTED: u32   = !EXT2_FEATURE_RO_COMPAT_SUPP;

const EXT3_FEATURE_RO_COMPAT_SUPP: u32          = (EXT2_FEATURE_RO_COMPAT_SPARSE_SUPER | EXT2_FEATURE_RO_COMPAT_LARGE_FILE | EXT2_FEATURE_RO_COMPAT_BTREE_DIR);
const EXT3_FEATURE_INCOMPAT_SUPP: u32           = (EXT2_FEATURE_INCOMPAT_FILETYPE | EXT3_FEATURE_INCOMPAT_RECOVER | EXT2_FEATURE_INCOMPAT_META_BG);
const EXT3_FEATURE_INCOMPAT_UNSUPPORTED: u32	= !EXT3_FEATURE_INCOMPAT_SUPP;
const EXT3_FEATURE_RO_COMPAT_UNSUPPORTED: u32   = !EXT3_FEATURE_RO_COMPAT_SUPP;

#[repr (C, packed)]
pub struct ExtSuperBlock {
    inodes_count: u32,
    blocks_count: u32,
    reserved_blocks_count: u32,
    n_unallocated_blocks: u32,
    n_unallocated_inodes: u32,
    first_data_block: u32,
    log_block_size: u32,
    log_fragment_size: u32,
    n_blocks_per_group: u32,
    n_fragments_per_group: u32,
    n_inodes_per_group: u32,
    last_mount_time: u32,
    last_write_time: u32,
    n_mounts: u16,
    n_mounts_before_fsck: u16,
    signature: u16,
    state: u16,
    error_behaviour: u16,
    minor_version: u16,
    last_fsck: u32,
    time_between_fscks: u32,
    os_id: u32,
    major_version: u32,
    superuser_uid: u16,
    superuser_gid: u16,
    extended: ExtExtendedSuperBlock
}

#[repr (C, packed)]
struct ExtExtendedSuperBlock {
    first_ino: u32,
    ino_size: u16,
    block_group: u16,
    feature_compat: u32,
    feature_incompat: u32,
    feature_ro_compat: u32,
    uuid: [u8;16],
    label: [u8;16],
    last_mount_path: [u8;64],
    algorithm_usage_bitmap: u32,
    preallocated_blocks: u8,
    preallocated_dirs: u8,
    unused: u16,
    journal_uuid: [u8;16],
    journal_inode: u32,
    journal_device: u32,
    orphans_head: u32
}

impl OffsetSuperBlock for ExtSuperBlock{
    fn get_superblock_offset() -> u64 {
        return EXT_SB_OFF;
    }
}

impl SuperBlock for ExtSuperBlock {
    fn get_label(&self) -> Option<String> {
        if let Ok(cstr) = cstr_to_string(&self.extended.label) {
            return Some(cstr);
        }
        return None;
    }

    fn get_uuid(&self) -> Option<UUID> {
        return Some(UUID::from_slice16(self.extended.uuid));
    }

    fn validate_superblock(&self) -> bool {
        if self.signature != EXT_MAGIC {
            return false;
        }

        if self.extended.feature_compat & EXT3_FEATURE_COMPAT_HAS_JOURNAL != 0 {
            return false;
        }

        if (self.extended.feature_ro_compat & EXT2_FEATURE_RO_COMPAT_UNSUPPORTED != 0) || (self.extended.feature_incompat & EXT2_FEATURE_INCOMPAT_UNSUPPORTED != 0) {
            return false;
        }
        
        return true;
    }
}

#[repr (C, packed)]
pub struct Ext3SuperBlock {
    block: ExtSuperBlock
}

impl OffsetSuperBlock for Ext3SuperBlock{
    fn get_superblock_offset() -> u64 {
        return EXT_SB_OFF;
    }
}

impl SuperBlock for Ext3SuperBlock {
    fn get_label(&self) -> Option<String> {
        if let Ok(cstr) = cstr_to_string(&self.block.extended.label) {
            return Some(cstr);
        }
        return None;
    }

    fn get_uuid(&self) -> Option<UUID> {
        return Some(UUID::from_slice16(self.block.extended.uuid));
    }

    fn validate_superblock(&self) -> bool {
        if self.block.signature != EXT_MAGIC {
            return false;
        }

        if self.block.extended.feature_compat & EXT3_FEATURE_COMPAT_HAS_JOURNAL == 0 {
            return false;
        }

        if (self.block.extended.feature_ro_compat & EXT3_FEATURE_RO_COMPAT_UNSUPPORTED != 0) || (self.block.extended.feature_incompat & EXT3_FEATURE_INCOMPAT_UNSUPPORTED != 0) {
            return false;
        }
        
        return true;
    }
}

#[repr (C, packed)]
pub struct Ext4SuperBlock {
    block: ExtSuperBlock,
    hash_seed: [u32;4],
    hash_version: u8,
    journal_backup_type: u8,
    desc_size: u16,
    default_mount_opts: u32,
    first_meta_block_group: u32,
    mkfs_time: u32,
    journal_blocks: [u32;17],
    block_count_high: u32,
    reserved_block_count_high: u32,
    free_blocks_count_high: u32,
    min_extra_isize: u16,
    want_extra_isize: u16,
    flags: u32,
    raid_stride: u16,
    mmp_interval: u16,
    mmp_block: u64,
    raid_stripe_width: u32,
    log_groups_per_flex: u8,
    checksum_type: u8,
    reserved: u16,
    kbytes_written: u64,
    snapshot_inum: u32,
    snapshot_id: u32,
    snapshot_r_blocks_count: u64,
    snapshot_list: u32,
    error_count: u32,
    first_error_time: u32,
    first_error_ino: u32,
    first_error_block: u64,
    first_error_func: [u8;32],
    first_error_line: u32,
    last_error_time: u32,
    last_error_ino: u32,
    last_error_line: u32,
    last_error_block: u32,
    last_error_fund: [u8;32],
    mount_opts: [u8;64],
    usr_quota_inum: u32,
    grp_quota_inum: u32,
    overhead_blocks: u32,
    backup_bgs: [u32;2],
    encryption_algos: [u8;4],
    password_salt: [u8;16],
    lpf_ino: u32,
    prj_quota_inum: u32,
    checksum_seed: u32,
    reserved2: [u32;98],
    checksum: u32
}

impl OffsetSuperBlock for Ext4SuperBlock{
    fn get_superblock_offset() -> u64 {
        return EXT_SB_OFF;
    }
}

impl SuperBlock for Ext4SuperBlock {
    fn get_label(&self) -> Option<String> {
        if let Ok(cstr) = cstr_to_string(&self.block.extended.label) {
            return Some(cstr);
        }
        return None;
    }

    fn get_uuid(&self) -> Option<UUID> {
        return Some(UUID::from_slice16(self.block.extended.uuid));
    }

    fn validate_superblock(&self) -> bool {
        if self.block.signature != EXT_MAGIC {
            return false;
        }

        if self.block.extended.feature_incompat & EXT3_FEATURE_INCOMPAT_JOURNAL_DEV != 0 {
            return false;
        }

        if (self.block.extended.feature_ro_compat & EXT3_FEATURE_RO_COMPAT_UNSUPPORTED == 0) && (self.block.extended.feature_incompat & EXT3_FEATURE_INCOMPAT_UNSUPPORTED == 0) {
            return false;
        }

        return true;
    }
}