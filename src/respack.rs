use std::ffi::OsStr;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use quick_xml::DeError;
use serde::{Deserialize, Serialize};
use zip::result::ZipError;
use zip::ZipArchive;

#[derive(Debug)]
pub struct Respack<'p> {
	path: &'p Path,
	data: RespackData,
}

#[derive(Debug)]
pub enum RespackError {
	IO(std::io::Error),
	Zip(ZipError),
	XML(DeError),
	MissingMetadata(String),
}

impl From<std::io::Error> for RespackError {
	fn from(err: std::io::Error) -> Self {
		RespackError::IO(err)
	}
}

impl From<ZipError> for RespackError {
	fn from(err: ZipError) -> Self {
		RespackError::Zip(err)
	}
}

impl From<DeError> for RespackError {
	fn from(err: DeError) -> Self {
		RespackError::XML(err)
	}
}

impl Respack<'_> {
	pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, RespackError> {
		let file = File::open(&path)?;
		let mut arc = ZipArchive::new(file)?;

		let mut info = None;

		for idx in 0..arc.len() {
			let entry = arc.by_index(idx)?;
			let name = entry.enclosed_name();

			if let Some(name) = name {
				match name.file_name() {
					Some(name) if name == OsStr::new("info.xml") => {
						info = Some(quick_xml::de::from_reader(BufReader::new(entry))?);
					}
					_ => {}
				}
			} else {
				println!(
					"Encountered invalid path: {:?}, continuing...",
					entry.mangled_name()
				);
			}
		}

		if info.is_none() {
			Err(RespackError::MissingMetadata(
				"Missing info.xml file in respack".to_string(),
			))
		} else {
			let data = RespackData {
				info: info.unwrap(),
			};
			Ok(Self {
				path: path.as_ref(),
				data,
			})
		}
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RespackData {
	info: RespackInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RespackInfo {
	name: String,
	author: Option<String>,
	description: Option<String>,
	link: Option<String>,
}
