use std::fmt;
use std::io;
use std::convert::TryFrom;
use io::Endianness;

#[derive(Debug)]
pub enum InvalidELFFormatError {
    InvalidMagic,
    InvalidEndianness(u8),
    InvalidAddressSize(u8),
    InvalidABI(u8),
    InvalidObjectType(u16),
    InvalidTargetArch(u16),
    InvalidVersion(u8),
    InvalidProgramHeaderEntryType(u32),
    InvalidSectionHeaderEntryType(u32),
    InvalidSectionHeaderEntryFlag(u64),
    InvalidSymbolType(u8),
    InvalidSymbolBinding(u8),
    InvalidSymbolVisibility(u8),
    MalformedSection,
    IOError(io::Error)
}

impl fmt::Display for InvalidELFFormatError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl InvalidELFFormatError {
    pub fn to_string(&self) -> String {
        match self {
            InvalidELFFormatError::InvalidMagic => format!("Invalid Magic Number"),
            InvalidELFFormatError::InvalidEndianness(e) => format!("Invalid Endianness. Expected 1 or 2, got {}", e),
            InvalidELFFormatError::InvalidAddressSize(e) => format!("Invalid Address Size. Expected 1 or 2, got {}", e),
            InvalidELFFormatError::InvalidABI(e) => format!("Invalid ABI. Got {}", e),
            InvalidELFFormatError::InvalidObjectType(e) => format!("Invalid Object Type. Got {}", e),
            InvalidELFFormatError::InvalidTargetArch(e) => format!("Invalid Target Architecture. Got {}", e),
            InvalidELFFormatError::InvalidVersion(e) => format!("Invalid ELF Version. Expected 1, got {}", e),
            InvalidELFFormatError::InvalidProgramHeaderEntryType(e) => format!("Invalid Program Header Entry Type. Got {}", e),
            InvalidELFFormatError::InvalidSectionHeaderEntryType(e) => format!("Invalid Section Header Entry Type. Got {}", e),
            InvalidELFFormatError::InvalidSectionHeaderEntryFlag(e) => format!("Invalid Section Header Flag. Got {}", e),
            InvalidELFFormatError::InvalidSymbolType(e) => format!("Invalid Symbol Type. Got {}", e),
            InvalidELFFormatError::InvalidSymbolBinding(e) => format!("Invalid Symbol Binding. Got {}", e),
            InvalidELFFormatError::InvalidSymbolVisibility(e) => format!("Invalid Symbol Visibility. Got {}", e),
            InvalidELFFormatError::MalformedSection => format!("Section was malformed"),
            InvalidELFFormatError::IOError(err) => err.to_string()
        }
    }
}

impl From<io::Error> for InvalidELFFormatError {
    fn from(err: io::Error) -> InvalidELFFormatError {
        return InvalidELFFormatError::IOError(err);
    }
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Clone, Copy)]
pub enum AddressSize {
    ThirtyTwoBit,
    SixtyFourBit,
}

impl TryFrom<u8> for AddressSize {
    type Error = InvalidELFFormatError;

    fn try_from(u: u8) -> Result<Self, Self::Error> {
        return match u {
            1 => Ok(AddressSize::ThirtyTwoBit),
            2 => Ok(AddressSize::SixtyFourBit),
            _ => Err(InvalidELFFormatError::InvalidAddressSize(u))
        }
    }
}

impl fmt::Display for AddressSize {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        return match self {
            AddressSize::ThirtyTwoBit => write!(f, "ELF32"),
            AddressSize::SixtyFourBit => write!(f, "ELF64")
        }
    }
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Clone, Copy)]
pub enum ElfEndianness{
    LittleEndian,
    BigEndian
}

impl TryFrom<u8> for ElfEndianness {
    type Error = InvalidELFFormatError;

    fn try_from(u: u8) -> Result<Self, Self::Error> {
        return match u {
            1 => Ok(ElfEndianness::LittleEndian),
            2 => Ok(ElfEndianness::BigEndian),
            _ => Err(InvalidELFFormatError::InvalidEndianness(u))
        }
    }
}

impl ElfEndianness {
    pub fn to_portable(&self) -> Endianness {
        return match self {
            ElfEndianness::LittleEndian => Endianness::Little,
            ElfEndianness::BigEndian => Endianness::Big
        }
    }
}

impl fmt::Display for ElfEndianness {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        return match self {
            ElfEndianness::LittleEndian => write!(f, "2's complement, little endian"),
            ElfEndianness::BigEndian => write!(f, "2's complement, big endian")
        }
    }
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Clone, Copy)]
#[allow(dead_code)]
pub enum ElfABI {
    SystemV,
    HpUx,
    NetBsd,
    Linux,
    GnuHurd,
    Solaris,
    AIX,
    IRIX,
    FreeBSD,
    Tru64,
    NovellModesto,
    OpenBSD,
    OpenVMS,
    NonStopKernel,
    AROS,
    FenixOS,
    CloudABI,
    OpenVOS,
}

impl TryFrom<u8> for ElfABI {
    type Error = InvalidELFFormatError;

    fn try_from(u: u8) -> Result<Self, Self::Error> {
        return match u {
            0x00 => Ok(ElfABI::SystemV),
            0x01 => Ok(ElfABI::HpUx),
            0x02 => Ok(ElfABI::NetBsd),
            0x03 => Ok(ElfABI::Linux),
            0x04 => Ok(ElfABI::GnuHurd),
            0x06 => Ok(ElfABI::Solaris),
            0x07 => Ok(ElfABI::AIX),
            0x08 => Ok(ElfABI::IRIX),
            0x09 => Ok(ElfABI::FreeBSD),
            0x0A => Ok(ElfABI::Tru64),
            0x0B => Ok(ElfABI::NovellModesto),
            0x0C => Ok(ElfABI::OpenBSD),
            0x0D => Ok(ElfABI::OpenVMS),
            0x0E => Ok(ElfABI::NonStopKernel),
            0x0F => Ok(ElfABI::AROS),
            0x10 => Ok(ElfABI::FenixOS),
            0x11 => Ok(ElfABI::CloudABI),
            0x12 => Ok(ElfABI::OpenVOS),
            _ => Err(InvalidELFFormatError::InvalidABI(u))
        }
    }
}

impl fmt::Display for ElfABI {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        return match self {
            ElfABI::SystemV => write!(f, "SystemV"),
            ElfABI::HpUx => write!(f, "UNIX - System V"),
            ElfABI::NetBsd => write!(f, "NetBSD"),
            ElfABI::Linux => write!(f, "Linux"),
            ElfABI::GnuHurd => write!(f, "GnuHurd"),
            ElfABI::Solaris => write!(f, "Solaris"),
            ElfABI::AIX => write!(f, "AIX"),
            ElfABI::IRIX => write!(f, "IRIX"),
            ElfABI::FreeBSD => write!(f, "FreeBSD"),
            ElfABI::Tru64 => write!(f, "Tru64"),
            ElfABI::NovellModesto => write!(f, "Novell Modesto"),
            ElfABI::OpenBSD => write!(f, "OpenBSD"),
            ElfABI::OpenVMS => write!(f, "OpenVMS"),
            ElfABI::NonStopKernel => write!(f, "Nonstop Kernel"),
            ElfABI::AROS => write!(f, "AROS"),
            ElfABI::FenixOS => write!(f, "FenixOS"),
            ElfABI::CloudABI => write!(f, "CloudABI"),
            ElfABI::OpenVOS => write!(f, "OpenVOS")
        }
    }
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
#[allow(dead_code)]
pub enum ElfObjectType {
    None,
    Relocatable,
    Executable,
    SharedObject,
    Core,
    OsSpecific(u16),
    ProcessSpecific(u16)
}

const ET_LOOS: u16 = 0xFE00;
const ET_HIOS: u16 = 0xFEFF;
const ET_LOPROC: u16 = 0xFF00;
const ET_HIPROC: u16 = 0xFFFF;

impl TryFrom<u16> for ElfObjectType {
    type Error = InvalidELFFormatError;

    fn try_from(u: u16) -> Result<Self, Self::Error> {
        match u {
            0x00 => Ok(ElfObjectType::None),
            0x01 => Ok(ElfObjectType::Relocatable),
            0x02 => Ok(ElfObjectType::Executable),
            0x03 => Ok(ElfObjectType::SharedObject),
            0x04 => Ok(ElfObjectType::Core),
            a if a >= ET_LOOS && a <= ET_HIOS => Ok(ElfObjectType::OsSpecific(a)),
            a if a >= ET_LOPROC && a <= ET_HIPROC => Ok(ElfObjectType::ProcessSpecific(a)),
            _ => Err(InvalidELFFormatError::InvalidObjectType(u))
        }
    }
}

impl fmt::Display for ElfObjectType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        return match self {
            ElfObjectType::None => write!(f, "None"),
            ElfObjectType::Relocatable => write!(f, "Relocatable"),
            ElfObjectType::Executable => write!(f, "EXE (Executable File)"),
            ElfObjectType::SharedObject => write!(f, "DYN (Shared Object)"),
            ElfObjectType::Core => write!(f, "Core"),
            ElfObjectType::OsSpecific(a) => write!(f, "OS Specific - {}", a),
            ElfObjectType::ProcessSpecific(a) => write!(f, "Process Specific - {}", a),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
#[allow(dead_code)]
pub enum ElfTargetArch {
    None,
    WE32100,
    Sparc,
    Intelx86,
    Motorolla68000,
    Motorolla88000,
    IntelMCU,
    Intel80860,
    MIPS,
    IBM370,
    MipsLittleEndian,
    HpPaRISC,
    Intel80960,
    PowerPC,
    PowerPC64,
    S390,
    ARM,
    SuperH,
    IA64,
    AMD64,
    ARM64,
    RiscV
}

impl TryFrom<u16> for ElfTargetArch {
    type Error = InvalidELFFormatError;

    fn try_from(u: u16) -> Result<Self, Self::Error> {
        match u {
            0x00 => Ok(ElfTargetArch::None),
            0x01 => Ok(ElfTargetArch::WE32100),
            0x02 => Ok(ElfTargetArch::Sparc),
            0x03 => Ok(ElfTargetArch::Intelx86),
            0x04 => Ok(ElfTargetArch::Motorolla68000),
            0x05 => Ok(ElfTargetArch::Motorolla88000),
            0x06 => Ok(ElfTargetArch::IntelMCU),
            0x07 => Ok(ElfTargetArch::Intel80860),
            0x08 => Ok(ElfTargetArch::MIPS),
            0x09 => Ok(ElfTargetArch::IBM370),
            0x0A => Ok(ElfTargetArch::MipsLittleEndian),
            0x0E => Ok(ElfTargetArch::HpPaRISC),
            0x13 => Ok(ElfTargetArch::Intel80960),
            0x14 => Ok(ElfTargetArch::PowerPC),
            0x15 => Ok(ElfTargetArch::PowerPC64),
            0x16 => Ok(ElfTargetArch::S390),
            0x28 => Ok(ElfTargetArch::ARM),
            0x2A => Ok(ElfTargetArch::SuperH),
            0x32 => Ok(ElfTargetArch::IA64),
            0x3E => Ok(ElfTargetArch::AMD64),
            0xB7 => Ok(ElfTargetArch::ARM64),
            0xF3 => Ok(ElfTargetArch::RiscV),
            _ => Err(InvalidELFFormatError::InvalidTargetArch(u))
        }
    }
}

impl fmt::Display for ElfTargetArch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        return match self {
            ElfTargetArch::None => write!(f, "None"),
            ElfTargetArch::WE32100 => write!(f, "Western Electric 32100"),
            ElfTargetArch::Sparc => write!(f, "Sparc"),
            ElfTargetArch::Intelx86 => write!(f, "Intel x86"),
            ElfTargetArch::Motorolla68000 => write!(f, "Motorolla 68000"),
            ElfTargetArch::Motorolla88000 => write!(f, "Motorolla 88000"),
            ElfTargetArch::IntelMCU => write!(f, "Intel Quark Microcontroller"),
            ElfTargetArch::Intel80860 => write!(f, "Intel i860"),
            ElfTargetArch::MIPS => write!(f, "MIPS (Big Endian)"),
            ElfTargetArch::IBM370 => write!(f, "IBM System/370"),
            ElfTargetArch::MipsLittleEndian => write!(f, "MIPS (Little Endian)"),
            ElfTargetArch::HpPaRISC => write!(f, "HP PA-RISC"),
            ElfTargetArch::Intel80960 => write!(f, "Intel i960"),
            ElfTargetArch::PowerPC => write!(f, "PowerPC"),
            ElfTargetArch::PowerPC64 => write!(f, "PowerPC 64"),
            ElfTargetArch::S390 => write!(f, "IBM System/390"),
            ElfTargetArch::ARM => write!(f, "ARM"),
            ElfTargetArch::SuperH => write!(f, "RISC SuperH"),
            ElfTargetArch::IA64 => write!(f, "IA-64"),
            ElfTargetArch::AMD64 => write!(f, "AMD64"),
            ElfTargetArch::ARM64 => write!(f, "ARM64"),
            ElfTargetArch::RiscV => write!(f, "RiscV"),
        }
    }
}

/// This enum represents the "type" of a Program Segment in an ELF executable
#[allow(dead_code)]
#[derive(Debug)]
pub enum ProgramHeaderEntryType {
    /// Indicates this entry is unused
    None,

    /// Indicates a loadable segment
    Loadable,

    /// Indicates this entry contains information for dynamic linking
    DynamicLinkingInfo,

    /// 
    InterpreterInfo,
    AuxInfo,

    /// Entry containing the program header itself
    ProgramHeader,
    ThreadLocalStorage,

    OsSpecific(u32),
    ProcessorSpecific(u32)
}

/// The minimum value for OS specific Program Segment Types
const PT_LOOS: u32 = 0x60000000;

/// The maximum value for OS specific Program Segment Types
const PT_HIOS: u32 = 0x6FFFFFFF;

/// The minimum value for Processor Specific Program Segment Types
const PT_LOPROC: u32 = 0x70000000;

/// The maximum value for Processor Specific Program Segment Types
const PT_HIPROC: u32 = 0x7FFFFFFF;

impl TryFrom<u32> for ProgramHeaderEntryType {
    type Error = InvalidELFFormatError;

    fn try_from(u: u32) -> Result<Self, Self::Error> {
        match u {
            0x0 => Ok(ProgramHeaderEntryType::None),
            0x1 => Ok(ProgramHeaderEntryType::Loadable),
            0x2 => Ok(ProgramHeaderEntryType::DynamicLinkingInfo),
            0x3 => Ok(ProgramHeaderEntryType::InterpreterInfo),
            0x4 => Ok(ProgramHeaderEntryType::AuxInfo),
            0x6 => Ok(ProgramHeaderEntryType::ProgramHeader),
            0x7 => Ok(ProgramHeaderEntryType::ThreadLocalStorage),
            a if a >= PT_LOOS && a <= PT_HIOS => Ok(ProgramHeaderEntryType::OsSpecific(a)),
            a if a >= PT_LOPROC && a <= PT_HIPROC => Ok(ProgramHeaderEntryType::ProcessorSpecific(a)),
            _ => Err(InvalidELFFormatError::InvalidProgramHeaderEntryType(u))
        }
    }
}

// OS Specific Section Type Constants

/// The array element specifies the location and size of the exception handling information as defined by the .eh_frame_hdr section.
const PT_GNU_EH_FRAME: u32 = 0x6474e550;

/// The p_flags member specifies the permissions on the segment containing the stack and is used to indicate wether the stack should be executable. The absense of this header indicates that the stack will be executable.
const PT_GNU_STACK: u32 = 0x6474e551;

/// The array element specifies the location and size of a segment which may be made read-only after relocations have been processed.
const PT_GNU_RELRO: u32 = 0x6474e552;

/// The segment contains .note.gnu.property
const PT_GNU_PROPERTY: u32 = 0x6474e553;

impl fmt::Display for ProgramHeaderEntryType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        return match self {
            ProgramHeaderEntryType::None => write!(f, "NONE"),
            ProgramHeaderEntryType::Loadable => write!(f, "LOAD"),
            ProgramHeaderEntryType::DynamicLinkingInfo => write!(f, "DYNAMIC"),
            ProgramHeaderEntryType::InterpreterInfo => write!(f, "INTERP"),
            ProgramHeaderEntryType::AuxInfo => write!(f, "NOTE"),
            ProgramHeaderEntryType::ProgramHeader => write!(f, "PHDR"),
            ProgramHeaderEntryType::ThreadLocalStorage => write!(f, "TLS"),
            ProgramHeaderEntryType::OsSpecific(PT_GNU_EH_FRAME) => write!(f, "GNU EH Frame"),
            ProgramHeaderEntryType::OsSpecific(PT_GNU_STACK) => write!(f, "GNU Stack"),
            ProgramHeaderEntryType::OsSpecific(PT_GNU_RELRO) => write!(f, "Relocation Read-Only"),
            ProgramHeaderEntryType::OsSpecific(PT_GNU_PROPERTY) => write!(f, "GNU Property"),
            ProgramHeaderEntryType::OsSpecific(a) => write!(f, "OS Specific ({:x})", a),
            ProgramHeaderEntryType::ProcessorSpecific(a) => write!(f, "Processor Specific ({})", a)
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, PartialEq)]
pub enum SectionHeaderEntryType {
    None,
    ProgramData,
    SymbolTable,
    StringTable,
    RelocationTableAddends,
    SymbolHashTable,
    DynamicLinkingInfo,
    Notes,
    Blank,
    RelocationTable,
    DynamicLinkerSymbolTable,
    ConstructorArray,
    DestructorArray,
    PreConstructorArray,
    SectionGroup,
    SectionIndices,
    OsSpecific(u32)
}

/// The minimum value for OS specific types
const SHT_LOOS: u32 = 0x60000000;

impl TryFrom<u32> for SectionHeaderEntryType {
    type Error = InvalidELFFormatError;

    fn try_from(u: u32) -> Result<Self, Self::Error> {
        match u {
            0x0 => Ok(SectionHeaderEntryType::None),
            0x1 => Ok(SectionHeaderEntryType::ProgramData),
            0x2 => Ok(SectionHeaderEntryType::SymbolTable),
            0x3 => Ok(SectionHeaderEntryType::StringTable),
            0x4 => Ok(SectionHeaderEntryType::RelocationTableAddends),
            0x5 => Ok(SectionHeaderEntryType::SymbolHashTable),
            0x6 => Ok(SectionHeaderEntryType::DynamicLinkingInfo),
            0x7 => Ok(SectionHeaderEntryType::Notes),
            0x8 => Ok(SectionHeaderEntryType::Blank),
            0x9 => Ok(SectionHeaderEntryType::RelocationTable),
            0xB => Ok(SectionHeaderEntryType::DynamicLinkerSymbolTable),
            0xE => Ok(SectionHeaderEntryType::ConstructorArray),
            0xF => Ok(SectionHeaderEntryType::DestructorArray),
            0x10 => Ok(SectionHeaderEntryType::PreConstructorArray),
            0x11 => Ok(SectionHeaderEntryType::SectionGroup),
            0x12 => Ok(SectionHeaderEntryType::SectionIndices),
            a if a >= SHT_LOOS => Ok(SectionHeaderEntryType::OsSpecific(a)),
            _ => Err(InvalidELFFormatError::InvalidSectionHeaderEntryType(u))
        }
    }
}

/// Incremental Build Data
const SHT_GNU_INCREMENTAL_INPUTS: u32 = 0x6fff4700;

/// LLVM ODR table
const SHT_LLVM_ODRTAB: u32 = 0x6fff4c00;

/// Object attributes
const SHT_GNU_ATTRIBUTES: u32 = 0x6ffffff5;

/// GNU style symbol hash table
const SHT_GNU_HASH: u32 = 0x6ffffff6;

/// List of prelink dependencies
const SHT_GNU_LIBLIST: u32 = 0x6ffffff7;

impl fmt::Display for SectionHeaderEntryType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        return match self {
            SectionHeaderEntryType::None => write!(f, "NULL"),
            SectionHeaderEntryType::ProgramData => write!(f, "PROGBITS"),
            SectionHeaderEntryType::SymbolTable => write!(f, "DYNSYM"),
            SectionHeaderEntryType::StringTable => write!(f, "STRTAB"),
            SectionHeaderEntryType::RelocationTableAddends => write!(f, "RELA"),
            SectionHeaderEntryType::SymbolHashTable => write!(f, "HASH"),
            SectionHeaderEntryType::DynamicLinkingInfo => write!(f, "DYNAMIC"),
            SectionHeaderEntryType::Notes => write!(f, "NOTE"),
            SectionHeaderEntryType::Blank => write!(f, "NOBITS"),
            SectionHeaderEntryType::RelocationTable => write!(f, "REL"),
            SectionHeaderEntryType::DynamicLinkerSymbolTable => write!(f, "DYNSYM"),
            SectionHeaderEntryType::ConstructorArray => write!(f, "INIT_ARRAY"),
            SectionHeaderEntryType::DestructorArray => write!(f, "FINI_ARRAY"),
            SectionHeaderEntryType::PreConstructorArray => write!(f, "PREINIT_ARRAY"),
            SectionHeaderEntryType::SectionGroup => write!(f, "GROUP"),
            SectionHeaderEntryType::SectionIndices => write!(f, "INDICES"),
            SectionHeaderEntryType::OsSpecific(SHT_GNU_INCREMENTAL_INPUTS) => write!(f, "GNU Incremental"),
            SectionHeaderEntryType::OsSpecific(SHT_LLVM_ODRTAB) => write!(f, "LLVM ODR Tab"),
            SectionHeaderEntryType::OsSpecific(SHT_GNU_ATTRIBUTES) => write!(f, "GNU Attributes"),
            SectionHeaderEntryType::OsSpecific(SHT_GNU_HASH) => write!(f, "GNU Hash Table"),
            SectionHeaderEntryType::OsSpecific(SHT_GNU_LIBLIST) => write!(f, "GNU Lib List"),
            SectionHeaderEntryType::OsSpecific(a) => write!(f, "OS Specific ({:x})", a),
        }
    }
}

#[allow(dead_code)]
bitflags! {
    pub struct SectionHeaderEntryFlags : u64 {
        const WRITABLE        = 0x1;
        const ALLOCATABLE     = 0x2;
        const EXECUTABLE      = 0x4;
        const MERGABLE        = 0x10;
        const STRINGS         = 0x20;
        const INFOLINK        = 0x40;
        const PRESERVEORDER   = 0x80;
        const NONCONFORMING   = 0x100;
        const GROUPMEMBER     = 0x200;
        const THREADLOCALDATA = 0x400;
    }
}

impl TryFrom<u64> for SectionHeaderEntryFlags {
    type Error = InvalidELFFormatError;

    fn try_from(u: u64) -> Result<Self, Self::Error> {
        return match SectionHeaderEntryFlags::from_bits(u) {
            Some(flags) => Ok(flags),
            None => Err(InvalidELFFormatError::InvalidSectionHeaderEntryFlag(u))
        }
    }
}

/// A "Type" of symbol from an ELF file
#[derive(PartialEq, Clone, Debug)]
pub enum SymType {
    /// Symbol's type is not specified
    NoType,

    /// Symbol is a data object (variable, array, etc.)
    Object,

    /// Symbol is executable code (function, etc.)
    Function,

    /// Symbol refers to a section
    Section,

    /// Local, absolute symbol that refers to a file
    File,

    /// An uninitialized common block
    CommonBlock,

    /// Thread local data object
    ThreadLocal,

    /// GNU indirect function
    GNUIndirectFunction,

    /// Operating System Specific Types
    OperatingSystemSpecific(u8),

    /// Processor Specific Symbols
    ProcessorSpecific(u8),
}

impl TryFrom<u8> for SymType {
    type Error = InvalidELFFormatError;

    fn try_from(u: u8) -> Result<Self, Self::Error> {
        match u {
            0 => Ok(SymType::NoType),
            1 => Ok(SymType::Object),
            2 => Ok(SymType::Function),
            3 => Ok(SymType::Section),
            4 => Ok(SymType::File),
            5 => Ok(SymType::CommonBlock),
            6 => Ok(SymType::ThreadLocal),
            10 => Ok(SymType::GNUIndirectFunction),
            11 | 12 => Ok(SymType::OperatingSystemSpecific(u)),
            13 | 14 | 15 => Ok(SymType::ProcessorSpecific(u)),
            _ => Err(InvalidELFFormatError::InvalidSymbolType(u))
        }
    }
}

impl fmt::Display for SymType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SymType::NoType =>   write!(f, "NoType"),
            SymType::Object =>   write!(f, "Object"),
            SymType::Function => write!(f, "Function"),
            SymType::Section =>  write!(f, "Section"),
            SymType::File =>     write!(f, "File"),
            SymType::CommonBlock => write!(f, "Common"),
            SymType::ThreadLocal => write!(f, "TLS"),
            SymType::GNUIndirectFunction => write!(f, "Indirect"),
            SymType::OperatingSystemSpecific(u) => write!(f, "OS({})", u),
            SymType::ProcessorSpecific(u) => write!(f, "Proc({})", u),
        }
    }
}

/// Represents the Binding (Visibility to other Object files loaded) of a given Symbol
#[derive(PartialEq, Clone, Debug)]
pub enum SymBinding {
    /// Local symbol, not visible outside obj file containing def
    Local,

    /// Global symbol, visible to all object files being combined
    Global,

    /// Weak symbol, like global but lower-precedence
    Weak,

    GNUUnique,

    /// Operating System Specific Bindings
    OperatingSystemSpecific(u8),

    /// Processor Specific Bindings
    ProcessorSpecific(u8)
}

impl TryFrom<u8> for SymBinding {
    type Error = InvalidELFFormatError;

    fn try_from(u: u8) -> Result<Self, Self::Error> {
        match u {
            0 => Ok(SymBinding::Local),
            1 => Ok(SymBinding::Global),
            2 => Ok(SymBinding::Weak),
            10 => Ok(SymBinding::GNUUnique),
            11 |12 => Ok(SymBinding::OperatingSystemSpecific(u)),
            13 | 14 | 15 => Ok(SymBinding::ProcessorSpecific(u)),
            _ => Err(InvalidELFFormatError::InvalidSymbolBinding(u))
        }
    }
}

impl fmt::Display for SymBinding {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SymBinding::Local => write!(f, "Local"),
            SymBinding::Global => write!(f, "Global"),
            SymBinding::Weak => write!(f, "Weak"),
            SymBinding::GNUUnique => write!(f, "Unique"),
            SymBinding::OperatingSystemSpecific(u) => write!(f, "OS({})", u),
            SymBinding::ProcessorSpecific(u) => write!(f, "Proc({})", u),
        }
    }
}

/// Defines how a given symbol may be accessed once the symbol has become part of an executable or shared object.
/// See: https://docs.oracle.com/cd/E19683-01/816-1386/6m7qcoblj/index.html#chapter7-27
#[derive(PartialEq, Clone, Debug)]
pub enum SymVisibility {
    /// Visibility is specified by binding type
    Default,

    /// Defined by processor supplements
    Internal,

    /// Not visible to other components
    Hidden,

    /// Visible in other components but not preemptable
    Protected,
}

impl TryFrom<u8> for SymVisibility {
    type Error = InvalidELFFormatError;

    fn try_from(u: u8) -> Result<Self, Self::Error> {
        return match u {
            0 => Ok(SymVisibility::Default),
            1 => Ok(SymVisibility::Internal),
            2 => Ok(SymVisibility::Hidden),
            3 => Ok(SymVisibility::Protected),
            _ => Err(InvalidELFFormatError::InvalidSymbolVisibility(u))
        }
    }
}

impl fmt::Display for SymVisibility {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SymVisibility::Default => write!(f, "Default"),
            SymVisibility::Internal => write!(f, "Internal"),
            SymVisibility::Hidden => write!(f, "Hidden"),
            SymVisibility::Protected => write!(f, "Protected")
        }
    }
}
  