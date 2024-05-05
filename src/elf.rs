use std::{
	fs::File,
	io::{self, BufReader, Read, Seek, SeekFrom},
};

use crate::error::Error;

type Result<T> = std::result::Result<T, Error>;

pub struct Elf {
	pub header: FileHeader,
	pub segments: Vec<Segment>,
	pub sections: Vec<Section>,
}

pub struct FileHeader {
	pub ident: Ident,
	pub kind: FileType,
	pub machine: Machine,
	pub version: u32,
	pub entry: u64,
	pub program_header_offset: u64,
	pub section_header_offset: u64,
	pub flags: u32,
	pub header_size: u16,
	pub program_header_entry_size: u16,
	pub program_header_count: u16,
	pub section_header_entry_size: u16,
	pub section_header_count: u16,
	pub section_header_names_index: u16,
}

pub struct Ident {
	pub magic: u32,
	pub is_64_bit: bool,
	pub is_little_endian: bool,
	pub version: u8,
	pub os_abi: OsAbi,
	pub abi_version: u8,
}

#[derive(Debug, PartialEq)]
pub enum OsAbi {
	SystemV,
	Linux,
	Other(u8),
}

impl From<u8> for OsAbi {
	fn from(value: u8) -> Self {
		match value {
			0x0 => Self::SystemV,
			0x3 => Self::Linux,
			_ => Self::Other(value),
		}
	}
}

impl From<OsAbi> for u8 {
	fn from(os_abi: OsAbi) -> Self {
		match os_abi {
			OsAbi::SystemV => 0x0,
			OsAbi::Linux => 0x3,
			OsAbi::Other(value) => value,
		}
	}
}

#[derive(Debug, PartialEq)]
pub enum FileType {
	None,
	Relocatable,
	Executable,
	Dynamic,
	Core,
	OperatingSystem(u16),
	Processor(u16),
	Other(u16),
}

impl From<u16> for FileType {
	fn from(value: u16) -> Self {
		match value {
			0x0 => Self::None,
			0x1 => Self::Relocatable,
			0x2 => Self::Executable,
			0x3 => Self::Dynamic,
			0x4 => Self::Core,
			0x5..=0xFDFF => Self::Other(value),
			0xFE00..=0xFEFF => Self::OperatingSystem(value),
			0xFF00..=0xFFFF => Self::Processor(value),
		}
	}
}

impl From<FileType> for u16 {
	fn from(file_type: FileType) -> Self {
		match file_type {
			FileType::None => 0x0,
			FileType::Relocatable => 0x1,
			FileType::Executable => 0x2,
			FileType::Dynamic => 0x3,
			FileType::Core => 0x4,
			FileType::Other(value) => value,
			FileType::OperatingSystem(value) => value,
			FileType::Processor(value) => value,
		}
	}
}

#[derive(Debug, PartialEq)]
pub enum Machine {
	X86,
	Arm,
	Amd64,
	Arm64,
	Other(u16),
}

impl From<u16> for Machine {
	fn from(value: u16) -> Self {
		match value {
			0x03 => Self::X86,
			0x28 => Self::Arm,
			0x3E => Self::Amd64,
			0xB7 => Self::Arm64,
			_ => Self::Other(value),
		}
	}
}

impl From<Machine> for u16 {
	fn from(machine: Machine) -> Self {
		match machine {
			Machine::X86 => 0x03,
			Machine::Arm => 0x28,
			Machine::Amd64 => 0x3E,
			Machine::Arm64 => 0xB7,
			Machine::Other(value) => value,
		}
	}
}

pub struct Segment {
	pub kind: ProgramType,
	pub flags: u32,
	pub offset: u64,
	pub virtual_address: u64,
	pub physical_address: u64,
	pub file_size: u64,
	pub memory_size: u64,
	pub alignment: u64,
	pub data: Vec<u8>,
}

#[derive(Debug, PartialEq)]
pub enum ProgramType {
	Null,
	Load,
	Dynamic,
	Interpreter,
	Note,
	SharedLibrary,
	ProgramHeaders,
	ThreadLocalStorage,
	OperatingSystem(u32),
	Processor(u32),
	Other(u32),
}

impl From<u32> for ProgramType {
	fn from(value: u32) -> Self {
		match value {
			0x0 => Self::Null,
			0x1 => Self::Load,
			0x2 => Self::Dynamic,
			0x3 => Self::Interpreter,
			0x4 => Self::Note,
			0x5 => Self::SharedLibrary,
			0x6 => Self::ProgramHeaders,
			0x7 => Self::ThreadLocalStorage,
			0x8..=0x5FFF_FFFF => Self::Other(value),
			0x6000_0000..=0x6FFF_FFFF => Self::OperatingSystem(value),
			0x7000_0000..=0x7FFF_FFFF => Self::Processor(value),
			0x8000_0000..=0xFFFF_FFFF => Self::Other(value),
		}
	}
}

impl From<ProgramType> for u32 {
	fn from(program_type: ProgramType) -> u32 {
		match program_type {
			ProgramType::Null => 0x0,
			ProgramType::Load => 0x1,
			ProgramType::Dynamic => 0x2,
			ProgramType::Interpreter => 0x3,
			ProgramType::Note => 0x4,
			ProgramType::SharedLibrary => 0x5,
			ProgramType::ProgramHeaders => 0x6,
			ProgramType::ThreadLocalStorage => 0x7,
			ProgramType::Other(value) => value,
			ProgramType::OperatingSystem(value) => value,
			ProgramType::Processor(value) => value,
		}
	}
}

pub struct Section {
	pub name_index: usize,
	pub kind: SectionType,
	pub flags: u64,
	pub address: u64,
	pub offset: u64,
	pub size: u64,
	pub link: u32,
	pub info: u32,
	pub address_alignment: u64,
	pub entry_size: u64,
	pub data: Vec<u8>,
}

#[derive(Debug, PartialEq)]
pub enum SectionType {
	Null,
	ProgramData,
	SymbolTable,
	StringTable,
	RelocationEntriesWithAddends,
	HashTable,
	Dynamic,
	Note,
	NoBits,
	RelocationEntries,
	SharedLibrary,
	LinkerSymbolTable,
	InitArray,
	FinishArray,
	PreInitArray,
	Group,
	SymbolTableNameIndices,
	OperatingSystem(u32),
	Processor(u32),
	Other(u32),
}

impl From<u32> for SectionType {
	fn from(value: u32) -> Self {
		match value {
			0x00 => Self::Null,
			0x01 => Self::ProgramData,
			0x02 => Self::SymbolTable,
			0x03 => Self::StringTable,
			0x04 => Self::RelocationEntriesWithAddends,
			0x05 => Self::HashTable,
			0x06 => Self::Dynamic,
			0x07 => Self::Note,
			0x08 => Self::NoBits,
			0x09 => Self::RelocationEntries,
			0x0A => Self::SharedLibrary,
			0x0B => Self::LinkerSymbolTable,
			0x0C..=0x0D => Self::Other(value),
			0x0E => Self::InitArray,
			0x0F => Self::FinishArray,
			0x10 => Self::PreInitArray,
			0x11 => Self::Group,
			0x12 => Self::SymbolTableNameIndices,
			0x13..=0x5FFF_FFFF => Self::Other(value),
			0x6000_0000..=0x6FFF_FFFF => Self::OperatingSystem(value),
			0x7000_0000..=0x7FFF_FFFF => Self::Processor(value),
			0x8000_0000..=0xFFFF_FFFF => Self::Other(value),
		}
	}
}

impl From<SectionType> for u32 {
	fn from(section_type: SectionType) -> u32 {
		match section_type {
			SectionType::Null => 0x00,
			SectionType::ProgramData => 0x01,
			SectionType::SymbolTable => 0x02,
			SectionType::StringTable => 0x03,
			SectionType::RelocationEntriesWithAddends => 0x04,
			SectionType::HashTable => 0x05,
			SectionType::Dynamic => 0x06,
			SectionType::Note => 0x07,
			SectionType::NoBits => 0x08,
			SectionType::RelocationEntries => 0x09,
			SectionType::SharedLibrary => 0x0A,
			SectionType::LinkerSymbolTable => 0x0B,
			SectionType::InitArray => 0x0E,
			SectionType::FinishArray => 0x0F,
			SectionType::PreInitArray => 0x10,
			SectionType::Group => 0x11,
			SectionType::SymbolTableNameIndices => 0x12,
			SectionType::OperatingSystem(value) => value,
			SectionType::Processor(value) => value,
			SectionType::Other(value) => value,
		}
	}
}

impl Elf {
	pub fn parse(file: File) -> Result<Elf> {
		let mut reader = ElfFile::new(file);

		let header = reader.read_header()?;
		let segments = reader
			.read_segments(header.program_header_offset, header.program_header_count.into())?;
		let sections = reader
			.read_sections(header.section_header_offset, header.section_header_count.into())?;

		Ok(Elf {
			header,
			segments,
			sections,
		})
	}
}

const ELF_CLASS_64: u8 = 0x02;
const ELF_DATA_LE: u8 = 0x01;

pub struct ElfFile {
	is_little_endian: bool,
	inner: BufReader<File>,
}

impl Seek for ElfFile {
	fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
		self.inner.seek(pos)
	}
}

impl ElfFile {
	fn new(file: File) -> Self {
		ElfFile {
			is_little_endian: true,
			inner: BufReader::new(file),
		}
	}

	fn read_header(&mut self) -> Result<FileHeader> {
		Ok(FileHeader {
			ident: self.read_ident()?,
			kind: self.read_u16().map(FileType::from)?,
			machine: self.read_u16().map(Machine::from)?,
			version: self.read_u32()?,
			entry: self.read_u64()?,
			program_header_offset: self.read_u64()?,
			section_header_offset: self.read_u64()?,
			flags: self.read_u32()?,
			header_size: self.read_u16()?,
			program_header_entry_size: self.read_u16()?,
			program_header_count: self.read_u16()?,
			section_header_entry_size: self.read_u16()?,
			section_header_count: self.read_u16()?,
			section_header_names_index: self.read_u16()?,
		})
	}

	fn read_ident(&mut self) -> Result<Ident> {
		let magic = self.read_u32()?;
		let is_64_bit = self.read_u8()? == ELF_CLASS_64;

		if magic != 0x464C457F || !is_64_bit {
			return Err(Error::from("File format is not ELF64!"));
		}

		let is_little_endian = self.read_u8()? == ELF_DATA_LE;
		self.is_little_endian = is_little_endian;

		let version = self.read_u8()?;

		let os_abi = self.read_u8().map(OsAbi::from)?;
		if os_abi != OsAbi::SystemV {
			return Err(format!("OS {os_abi:?} does not match expected Unix System V").into());
		}

		let abi_version = self.read_u8()?;

		// ignore padding
		self.seek(SeekFrom::Current(7))?;

		Ok(Ident {
			magic,
			is_64_bit,
			is_little_endian,
			version,
			os_abi,
			abi_version,
		})
	}

	fn read_segments(&mut self, offset: u64, count: usize) -> io::Result<Vec<Segment>> {
		let mut segments = Vec::with_capacity(count);

		self.seek(SeekFrom::Start(offset))?;

		for _ in 0..count {
			let kind = self.read_u32().map(ProgramType::from)?;
			let flags = self.read_u32()?;
			let offset = self.read_u64()?;
			let virtual_address = self.read_u64()?;
			let physical_address = self.read_u64()?;
			let file_size = self.read_u64()?;
			let memory_size = self.read_u64()?;
			let alignment = self.read_u64()?;

			let previous_pos = self.stream_position()?;

			self.seek(SeekFrom::Start(offset))?;

			let mut data = vec![0; file_size as usize];
			self.inner.read_exact(data.as_mut_slice())?;

			self.seek(SeekFrom::Start(previous_pos))?;

			segments.push(Segment {
				kind,
				flags,
				offset,
				virtual_address,
				physical_address,
				file_size,
				memory_size,
				alignment,
				data,
			});
		}

		Ok(segments)
	}

	fn read_sections(&mut self, offset: u64, count: usize) -> io::Result<Vec<Section>> {
		let mut sections = Vec::with_capacity(count);

		self.seek(SeekFrom::Start(offset))?;

		for _ in 0..count {
			let name_index = self.read_u32()? as usize;
			let kind = self.read_u32().map(SectionType::from)?;
			let flags = self.read_u64()?;
			let address = self.read_u64()?;
			let offset = self.read_u64()?;
			let size = self.read_u64()?;
			let link = self.read_u32()?;
			let info = self.read_u32()?;
			let address_alignment = self.read_u64()?;
			let entry_size = self.read_u64()?;

			let previous_pos = self.stream_position()?;

			self.seek(SeekFrom::Start(offset))?;

			let mut data = vec![0; size as usize];
			self.inner.read_exact(data.as_mut_slice())?;

			self.seek(SeekFrom::Start(previous_pos))?;

			sections.push(Section {
				name_index,
				kind,
				flags,
				address,
				offset,
				size,
				link,
				info,
				address_alignment,
				entry_size,
				data,
			});
		}

		Ok(sections)
	}

	fn read_u8(&mut self) -> io::Result<u8> {
		let mut buffer = [0; 1];
		self.inner.read_exact(&mut buffer)?;
		Ok(buffer[0])
	}

	fn read_u16(&mut self) -> io::Result<u16> {
		let mut buffer = [0; 2];
		self.inner.read_exact(&mut buffer)?;
		if self.is_little_endian {
			Ok(u16::from_le_bytes(buffer))
		} else {
			Ok(u16::from_be_bytes(buffer))
		}
	}

	fn read_u32(&mut self) -> io::Result<u32> {
		let mut buffer = [0; 4];
		self.inner.read_exact(&mut buffer)?;
		if self.is_little_endian {
			Ok(u32::from_le_bytes(buffer))
		} else {
			Ok(u32::from_be_bytes(buffer))
		}
	}

	fn read_u64(&mut self) -> io::Result<u64> {
		let mut buffer = [0; 8];
		self.inner.read_exact(&mut buffer)?;
		if self.is_little_endian {
			Ok(u64::from_le_bytes(buffer))
		} else {
			Ok(u64::from_be_bytes(buffer))
		}
	}
}
