use std::ffi::OsStr;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use quick_xml::DeError;
use serde::{Deserialize, Serialize};
use zip::result::ZipError;
use zip::ZipArchive;

#[derive(Debug)]
pub struct Respack {
	path: PathBuf,
	data: RespackData,
}

pub type RespackResult<T = ()> = Result<T, RespackError>;

#[derive(Debug)]
pub enum RespackError {
	Io(std::io::Error),
	Zip(ZipError),
	Xml(DeError),
	MissingMetadata(String),
}

impl From<std::io::Error> for RespackError {
	fn from(err: std::io::Error) -> Self {
		RespackError::Io(err)
	}
}

impl From<ZipError> for RespackError {
	fn from(err: ZipError) -> Self {
		RespackError::Zip(err)
	}
}

impl From<DeError> for RespackError {
	fn from(err: DeError) -> Self {
		RespackError::Xml(err)
	}
}

impl Respack {
	pub fn load_from_file<P: AsRef<Path>>(path: P) -> RespackResult<Self> {
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
				path: path.as_ref().to_path_buf(),
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
