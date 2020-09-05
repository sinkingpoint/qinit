extern crate libq;

use libq::elf::{AddressSize, ElfABI, ElfHeader, ElfObjectType, Endianness};
use libq::io::BufferReader;

#[test]
fn test_elf_header_reads_correctly() {
    // The first 64 bytes of an ELF header I pulled out of a random kernel module
    let header_bytes = [
        127, 69, 76, 70, 2, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 62, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 96,
        157, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 64, 0, 0, 0, 0, 0, 64, 0, 35, 0, 34, 0,
    ];
    let mut reader = BufferReader::new(&header_bytes);

    let header = match ElfHeader::read(&mut reader) {
        Ok(header) => header,
        Err(e) => {
            assert!(false, format!("Failed to read header: {:?}", e));
            return;
        }
    };

    assert!(header.is_valid());
    assert_eq!(
        header.addr_size,
        AddressSize::SixtyFourBit,
        "Failed to interpret header at a 64 bit ELF"
    );
    assert_eq!(header.endianness, Endianness::LittleEndian);
    assert_eq!(header.version, 1);
    assert_eq!(header.abi, ElfABI::SystemV); // SystemV (0) appears to be the default here for "Eh". Even though this is a linux kernel module
    assert_eq!(header.elf_type, ElfObjectType::Relocatable);
}
