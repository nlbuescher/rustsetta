mod elf;
mod error;

use std::fs::File;

use elf::Elf;
use error::Error;

pub fn main() -> Result<(), Error> {
	let mut args = std::env::args().skip(1);

	let path = args.next().ok_or("Filename argument not provided!")?;
	let elf = Elf::parse(File::open(path)?)?;

	println!("{:?} {:?}", elf.header.ident.os_abi, elf.header.machine);

	let names_index = elf.header.section_header_names_index;
	let name_data = &elf.sections[names_index as usize].data;

	for section in elf.sections.iter() {
		let start = section.name_index;
		let end = name_data[start..].iter().position(|it| *it == 0).unwrap() + start;
		let name = std::str::from_utf8(&name_data[start..end])
			.map_err(|_| "Failed to parse UTF8 string")?;

		println!("{name}: {start}, {end}");
	}

	Ok(())
}
