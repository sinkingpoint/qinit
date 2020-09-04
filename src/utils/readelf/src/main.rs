extern crate clap;
extern crate libq;

use std::path::Path;
use std::fs::File;
use std::process::exit;
use std::io::{Read, Seek};
use clap::{App, Arg};
use libq::elf::ElfBinary;
use libq::logger;

fn print_elf_header(binary: &ElfBinary) {
    println!("ELF Header: ");
    println!("  Magic:                             {:2x} {:2x} {:2x} {:2x}", binary.header.magic[0], binary.header.magic[1], binary.header.magic[2], binary.header.magic[3]);
    println!("  Class:                             {}", binary.header.addr_size);
    println!("  Data:                              {}", binary.header.endianness);
    println!("  OS/ABI:                            {}", binary.header.abi);
    println!("  ABI Version:                       {}", binary.header.abi_version);
    println!("  Type:                              {}", binary.header.elf_type);
    println!("  Machine:                           {}", binary.header.target_arch);
    println!("  Entrypoint Address:                {:<#15X}", binary.header.entrypoint);
    println!("  Start of program headers:          {} (bytes into the file)", binary.header.program_header_offset);
    println!("  Start of section headers:          {} (bytes into the file)", binary.header.section_header_table_offset);
    println!("  Flags:                             {:#x}", binary.header.flags);
    println!("  Size of this header:               {} (bytes)", binary.header.header_size);
    println!("  Size of program headers:           {} (bytes)", binary.header.program_header_entry_size);
    println!("  Number of program headers:         {}", binary.header.program_header_num_entries);
    println!("  Size of section headers:           {} (bytes)", binary.header.section_header_entry_size);
    println!("  Number of section headers:         {}", binary.header.section_header_num_entries);
    println!("  Section header string table index: {}", binary.header.section_header_name_index);
}

fn print_section_headers(binary: &ElfBinary) {
    println!("Section Headers:");
    println!("  {: <21} {: <25} {: <8} {: <8} {: <8} {: <8} {: <2} {: <2}", "Name", "Type", "Address", "Offset", "Size", "Entry Size", "Link", "Info");
    for section in binary.section_headers.iter() {
        println!("  {: <21} {: <25} {: <8} {: <8} {: <8} {: <8} {: <2} {: <2}", section.name, section.section_type, section.virtual_address, section.offset, section.size, section.entry_size, section.link_index, section.extra_info);
    }
}

fn print_program_headers(binary: &ElfBinary) {
    if binary.program_headers.len() == 0 {
        println!("There are no program headers in this file");
        return;
    }

    println!("Program Headers:");
    println!("  {: <21} {: <10} {: <8} {: <8} {: <8} {: <8} {: <2} {: <2}", "Type", "Offset", "VirtAddr", "PhysAddr", "FileSiz", "MemSiz", "Flags", "Align");
    for section in binary.program_headers.iter() {
        println!("  {: <21} {: <10} {: <8} {: <8} {: <8} {: <8} {: <2} {: <2}", section.section_type, section.offset, section.virtual_address, section.physical_address, section.file_size, section.mem_size, section.flags, section.alignment);
    }
}

fn print_symbols<T: Read + Seek>(binary: &ElfBinary, reader: &mut T) {
    let symbols = match binary.read_symbols(reader, ".symtab") {
        Ok(Some(symbols)) => symbols,
        _ => {
            println!("There are no symbols in this file");
            return;
        }
    };

    println!("Symbol Table .symtab contains {} entries", symbols.len());
    println!("{: >6} {: ^16} {: >6} {: ^12} {: ^8} {: ^8} {: ^4} {: >}", "Num", "Value", "Size", "Type", "Bind", "Vis", "NIdx", "Name");

    for (i, sym) in symbols.iter().enumerate() {
        let binding = match sym.get_binding() {
            Ok(b) => b,
            Err(e) => {
                println!("{: >6} {}", i, e);
                continue;
            }
        };

        let sym_type = match sym.get_type() {
            Ok(b) => b,
            Err(e) => {
                println!("{: >6} {}", i, e);
                continue;
            }
        };

        let visibility = match sym.get_visibility() {
            Ok(b) => b,
            Err(e) => {
                println!("{: >6} {}", i, e);
                continue;
            }
        };

        println!("{: >5}: {:0^16} {: >6} {:^12} {: ^8} {: ^8} {: ^4} {: >}", i, sym.value, sym.size, sym_type, binding, visibility, sym.name_index, sym.name);
    }
}

fn main() {
    let args = App::new("readelf")
        .version("0.1")
        .author("Colin D. <colin@quirl.co.nz>")
        .about("Prints information about a given ELF executable")
        .arg(Arg::with_name("all").short("a").help("Print all information available"))
        .arg(Arg::with_name("file-header").short("H").help("Print all information available"))
        .arg(Arg::with_name("program-headers").short("h").help("Print the files segment/program headers"))
        .arg(Arg::with_name("section-headers").short("S").help("Print the section headers"))
        .arg(Arg::with_name("symbols").short("s").help("Print the symbols"))
        .arg(
            Arg::with_name("file")
                .index(1)
                .help("The ELF file to read")
                .required(true),
        )
        .get_matches();

    let path = Path::new(args.value_of("file").unwrap());
    let logger = logger::with_name_as_console("readelf");
    if !path.exists() {
        logger
            .info()
            .with_str("path", args.value_of("file").unwrap())
            .smsg("Path doesn't exist");
        exit(1);
    }

    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(e) => {
            logger.info().with_str("path", args.value_of("file").unwrap()).with_string("error", e.to_string()).smsg("Failed to open file");
            exit(1);
        }
    };

    let binary = match ElfBinary::read(&mut file) {
        Ok(bin) => bin,
        Err(e) => {
            logger.info().with_str("path", args.value_of("file").unwrap()).with_string("error", e.to_string()).smsg("Failed to read file");
            exit(1);
        }
    };

    let all = args.is_present("all");
    let file_header = args.is_present("file-header") || all;
    let program_headers = args.is_present("program-headers") || all;
    let section_headers = args.is_present("section-headers") || all;
    let symbols = args.is_present("symbols") || all;

    if file_header {
        print_elf_header(&binary);
    }

    if program_headers {
        print_program_headers(&binary);
    }

    if section_headers {
        print_section_headers(&binary);
    }

    if symbols {
        print_symbols(&binary, &mut file);
    }
}