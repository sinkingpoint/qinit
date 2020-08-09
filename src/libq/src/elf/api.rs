use std::io::{self, Read, Seek, SeekFrom};
use std::convert::TryFrom;
use std::char;

use io::{Endianness, read_u8, read_u16, read_u32, read_u64};
use super::enums::{ProgramHeaderEntryType, SectionHeaderEntryType, SectionHeaderEntryFlags, ElfTargetArch, ElfObjectType, ElfABI, ElfEndianness, AddressSize, InvalidELFFormatError};

#[derive(Debug)]
pub struct ElfBinary {
    pub header: ElfHeader,
    pub program_headers: Vec<ProgramHeader>,
    pub section_headers: Vec<SectionHeader>
}

impl ElfBinary {
    pub fn read_section<T: Read + Seek>(&self, r: &mut T, name: &str) -> Result<Option<Vec<u8>>, io::Error> {
        for section in self.section_headers.iter() {
            if section.name == name {
                r.seek(SeekFrom::Start(section.offset))?;
                let mut buffer = vec![0; section.size];
                r.read_exact(&mut buffer)?;

                return Ok(Some(buffer));
            }
        }

        return Ok(None);
    }

    pub fn read<T: Read + Seek>(r: &mut T) -> Result<ElfBinary, InvalidELFFormatError> {
        let header = ElfHeader::read(r)?;

        if !header.is_valid() {
            return Err(InvalidELFFormatError::InvalidMagic);
        }

        let read_endian = header.endianness.to_portable();

        r.seek(SeekFrom::Start(header.program_header_offset))?;

        let mut program_headers = Vec::new();
        for _ in 0..header.program_header_num_entries {
            program_headers.push(ProgramHeader::read(r, header.addr_size, &read_endian)?);
        }

        r.seek(SeekFrom::Start(header.section_header_table_offset))?;

        let mut section_headers = Vec::new();
        for _ in 0..header.section_header_num_entries {
            section_headers.push(SectionHeader::read(r, header.addr_size, &read_endian)?);
        }

        // Lets dump out the names section
        let names_section_header = &section_headers[header.section_header_name_index]; // Assumes this is valid and we actually have a names section

        r.seek(SeekFrom::Start(names_section_header.offset))?;
        // TODO: We can tidy this up later. Probably by making a "StringsSection" struct
        let names_header_size = names_section_header.size;
        let mut buffer = vec![0; names_header_size];
        r.read_exact(&mut buffer)?;

        for header in section_headers.iter_mut() {
            let mut name_offset = header.name_offset;
            let mut build = String::new();
            while buffer[name_offset] != 0 && name_offset < names_header_size {
                build.push(char::from_u32(buffer[name_offset] as u32).unwrap());
                name_offset += 1;
            }

            header.name = build;
        }


        return Ok(ElfBinary {
            header: header,
            program_headers: program_headers,
            section_headers: section_headers
        });
    }
}

#[derive(Debug)]
pub struct ElfHeader {
    pub magic: [u8; 4],

    /// The Size of the address fields in the rest of the file
    pub addr_size: AddressSize,

    /// The Endianness of multibyte values in the rest of the file
    pub endianness: ElfEndianness,

    /// Target Platform ABI
    pub abi: ElfABI,

    /// Extra ABI information
    pub abi_version: u8,

    /// The Type of object this file represents
    pub elf_type: ElfObjectType,

    /// The Architecture this file was compiled for
    pub target_arch: ElfTargetArch,

    /// The ELF format version (Always 1. There's only 1 version)
    pub version: u32,

    /// The memory address of the entrypoint in this executable
    pub entrypoint: u64,

    /// The offset in this file of the program header
    pub program_header_offset: u64,

    /// The offset in this file of the section header
    pub section_header_table_offset: u64,

    /// Architecture dependant
    pub flags: u32,

    /// The size of this header
    pub header_size: u16,

    /// The size of an entry in the program header
    pub program_header_entry_size: u16,

    /// The number of entries in the program header
    pub program_header_num_entries: u16,

    /// The size of an entry in the section header
    pub section_header_entry_size: u16,

    /// The number of entries in the section header
    pub section_header_num_entries: u16,

    /// The index of the entry in the section header that contains the section names
    pub section_header_name_index: usize,
}

impl ElfHeader {
    pub fn is_valid(&self) -> bool {
        return self.magic == [0x7f, 0x45, 0x4c, 0x46];
    }

    pub fn read<T: Read>(mut r: &mut T) -> Result<ElfHeader, InvalidELFFormatError> {
        let mut magic = [0; 4];
        r.read_exact(&mut magic)?;

        let addr_size = AddressSize::try_from(read_u8(&mut r)?)?;
        let endianness = ElfEndianness::try_from(read_u8(&mut r)?)?;

        let read_endian = endianness.to_portable();

        let version = read_u8(&mut r)?;
        if version != 1 {
            return Err(InvalidELFFormatError::InvalidVersion(version));
        }

        let abi = ElfABI::try_from(read_u8(&mut r)?)?;
        let abi_version = read_u8(&mut r)?;

        let mut pad_buffer = [0; 7];
        r.read_exact(&mut pad_buffer)?;

        let elf_type = ElfObjectType::try_from(read_u16(&mut r, &read_endian)?)?;
        let target_arch = ElfTargetArch::try_from(read_u16(&mut r, &read_endian)?)?;

        // Two versions for some reason??
        let e_version = read_u32(&mut r, &read_endian)?;

        let entrypoint = match addr_size {
            AddressSize::ThirtyTwoBit => read_u32(&mut r, &read_endian)? as u64,
            AddressSize::SixtyFourBit => read_u64(&mut r, &read_endian)?
        };

        let program_header_offset = match addr_size {
            AddressSize::ThirtyTwoBit => read_u32(&mut r, &read_endian)? as u64,
            AddressSize::SixtyFourBit => read_u64(&mut r, &read_endian)?
        };

        let section_header_offset = match addr_size {
            AddressSize::ThirtyTwoBit => read_u32(&mut r, &read_endian)? as u64,
            AddressSize::SixtyFourBit => read_u64(&mut r, &read_endian)?
        };

        let flags = read_u32(&mut r, &read_endian)?;
        let header_size = read_u16(&mut r, &read_endian)?;
        let prog_header_size = read_u16(&mut r, &read_endian)?;
        let prog_header_num = read_u16(&mut r, &read_endian)?;
        let section_header_size = read_u16(&mut r, &read_endian)?;
        let section_header_num = read_u16(&mut r, &read_endian)?;
        let names_section_index = read_u16(&mut r, &read_endian)?;

        return Ok(
            ElfHeader{
                magic: magic,
                addr_size: addr_size,
                endianness: endianness,
                abi: abi,
                abi_version: abi_version,
                elf_type: elf_type,
                target_arch: target_arch,
                version: e_version,
                entrypoint: entrypoint,
                program_header_offset: program_header_offset,
                section_header_table_offset: section_header_offset,
                flags: flags,
                header_size: header_size,
                program_header_entry_size: prog_header_size,
                program_header_num_entries: prog_header_num,
                section_header_entry_size: section_header_size,
                section_header_num_entries: section_header_num,
                section_header_name_index: names_section_index as usize
            }
        )
    }
}

#[derive(Debug)]
pub struct ProgramHeader {
    /// Identifies the type of the segment. 
    pub section_type: ProgramHeaderEntryType,

    /// Segment dependant flags
    pub flags: u32,

    /// Offset of this segment in the image
    pub offset: u64,

    /// Virtual address of this segment in memory
    pub virtual_address: u64,

    /// Physical address of this segment in memory
    pub physical_address: u64,

    /// The size of this segment in the file
    pub file_size: u64,

    /// Size of this segment in memory (Can be different from the file size due to alignment etc)
    pub mem_size: u64,

    /// What power of two this segment should be aligned to
    pub alignment: u64
}

impl ProgramHeader {
    fn read<T: Read>(mut r: &mut T, addr_size: AddressSize, endianness: &Endianness) -> Result<ProgramHeader, InvalidELFFormatError> {
        // NOTE: Not sure whether the compiler can reorder these statements?
        // So this may be stochastic. Maybe.

        let section_type = ProgramHeaderEntryType::try_from(read_u32(r, endianness)?)?;

        // A bit hacky here - flags absolutely does only get initialized once, but the compiler can't reason that
        // so we have to give is a default value of 0 and make it mut

        let mut flags: u32 = 0;
        if addr_size == AddressSize::SixtyFourBit {
            // The flags are in a different position depending on the address size, for alignment reasons
            // For 64 bit addresses they're here, for 32 bit addresses, they're down
            flags = read_u32(r, endianness)?;
        }

        let segment_offset = match addr_size {
            AddressSize::ThirtyTwoBit => read_u32(&mut r, endianness)? as u64,
            AddressSize::SixtyFourBit => read_u64(&mut r, endianness)?
        };

        let virtual_address = match addr_size {
            AddressSize::ThirtyTwoBit => read_u32(&mut r, endianness)? as u64,
            AddressSize::SixtyFourBit => read_u64(&mut r, endianness)?
        };

        let physical_address = match addr_size {
            AddressSize::ThirtyTwoBit => read_u32(&mut r, endianness)? as u64,
            AddressSize::SixtyFourBit => read_u64(&mut r, endianness)?
        };

        let fs_size = match addr_size {
            AddressSize::ThirtyTwoBit => read_u32(&mut r, endianness)? as u64,
            AddressSize::SixtyFourBit => read_u64(&mut r, endianness)?
        };

        let mem_size = match addr_size {
            AddressSize::ThirtyTwoBit => read_u32(&mut r, endianness)? as u64,
            AddressSize::SixtyFourBit => read_u64(&mut r, endianness)?
        };

        if addr_size == AddressSize::ThirtyTwoBit {
            // The flags are in a different position depending on the address size, for alignment reasons
            // For 32 bit addresses they're here, for 64 bit addresses, they're above
            flags = read_u32(r, endianness)?;
        }

        let align = match addr_size {
            AddressSize::ThirtyTwoBit => read_u32(&mut r, endianness)? as u64,
            AddressSize::SixtyFourBit => read_u64(&mut r, endianness)?
        };

        return Ok(ProgramHeader {
            section_type: section_type,
            flags: flags,
            offset: segment_offset,
            virtual_address: virtual_address,
            physical_address: physical_address,
            file_size: fs_size,
            mem_size: mem_size,
            alignment: align
        });
    }
}

#[derive(Debug)]
pub struct SectionHeader {
    pub name: String,
    pub name_offset: usize,
    pub section_type: SectionHeaderEntryType,
    pub attrs: SectionHeaderEntryFlags,
    pub virtual_address: u64,
    pub offset: u64,
    pub size: usize,
    pub link_index: u32,
    pub extra_info: u32,
    pub align: u64,
    pub entry_size: u64
}

impl SectionHeader {
    pub fn read<T: Read>(mut r: &mut T, addr_size: AddressSize, endianness: &Endianness) -> Result<SectionHeader, InvalidELFFormatError> {
        let name_offset = read_u32(&mut r, endianness)?;
        let section_type = SectionHeaderEntryType::try_from(read_u32(&mut r, endianness)?)?;
        let flags = match addr_size {
            AddressSize::ThirtyTwoBit => SectionHeaderEntryFlags::try_from(read_u32(&mut r, endianness)? as u64)?,
            AddressSize::SixtyFourBit => SectionHeaderEntryFlags::try_from(read_u64(&mut r, endianness)?)?,
        };

        let virtual_address = match addr_size {
            AddressSize::ThirtyTwoBit => read_u32(&mut r, endianness)? as u64,
            AddressSize::SixtyFourBit => read_u64(&mut r, endianness)?
        };

        let offset = match addr_size {
            AddressSize::ThirtyTwoBit => read_u32(&mut r, endianness)? as u64,
            AddressSize::SixtyFourBit => read_u64(&mut r, endianness)?
        };

        let size = match addr_size {
            AddressSize::ThirtyTwoBit => read_u32(&mut r, endianness)? as u64,
            AddressSize::SixtyFourBit => read_u64(&mut r, endianness)?
        };

        let link_index = read_u32(&mut r, endianness)?;
        let extra_info = read_u32(&mut r, endianness)?;

        let align = match addr_size {
            AddressSize::ThirtyTwoBit => read_u32(&mut r, endianness)? as u64,
            AddressSize::SixtyFourBit => read_u64(&mut r, endianness)?
        };

        let entry_size = match addr_size {
            AddressSize::ThirtyTwoBit => read_u32(&mut r, endianness)? as u64,
            AddressSize::SixtyFourBit => read_u64(&mut r, endianness)?
        };

        return Ok(SectionHeader {
            name: String::new(),
            name_offset: name_offset as usize,
            section_type: section_type,
            attrs: flags,
            virtual_address: virtual_address,
            offset: offset,
            size: size as usize,
            link_index: link_index,
            extra_info: extra_info,
            align: align,
            entry_size: entry_size
        });
    }
}