extern crate regex;
extern crate serde;
extern crate serde_json;
extern crate unicase;
extern crate zip;

#[macro_use]
extern crate lazy_static;

use std::fs;
use std::result::Result;

use unicase::UniCase;

const IMPORT_DATA_DIRECTORY: &'static str = "data";

mod db;
use db::DB;

mod dict;

mod import;
use import::import_file;

fn main() {
	let start = std::time::Instant::now();

	// Validate the import data directory:
	let mut data_dir = std::env::current_dir().unwrap();
	data_dir.push(IMPORT_DATA_DIRECTORY);
	let data_dir_str = data_dir.to_string_lossy();
	let data_dir = match fs::metadata(&data_dir) {
		Ok(md) if md.is_dir() => {
			println!("\nImporting from {:}...", data_dir_str);
			data_dir
		}
		_ => {
			eprintln!("\nERROR: data directory not found at {:}\n", data_dir_str);
			std::process::exit(1);
		}
	};

	match import(data_dir) {
		Ok(_) => {
			println!("\nImporting finished after {:?}\n", start.elapsed());
		}
		Err(err) => {
			eprintln!("\nERROR: import failed: {:}\n", err);
			std::process::exit(2);
		}
	}
}

fn import<P: AsRef<std::path::Path>>(import_dir: P) -> Result<(), std::io::Error> {
	let start = std::time::Instant::now();
	let mut entries = Vec::new();
	for entry in fs::read_dir(import_dir)? {
		let entry = entry?;
		if entry.file_type()?.is_file() {
			let fullpath = entry.path();
			if let Some(ext) = fullpath.extension() {
				let ext = ext.to_string_lossy();
				if UniCase::new(ext) == UniCase::new("zip") {
					entries.push(fullpath);
				}
			}
		}
	}

	println!("Found {} file(s) to import...", entries.len());

	let mut db = DB::default();
	for fs in entries {
		let dict = import_file(fs)?;
		db.import_dict(dict);
	}

	db.finish_import();

	println!("\nImported database (elapsed {:?}):", start.elapsed());
	db.dump_info();

	let start = std::time::Instant::now();
	println!("\nExporting...");
	db.output()?;
	println!("... completed in {:?}", start.elapsed());

	Ok(())
}
