use ahash::{ RandomState, HashMapExt };
use crate::error::{ Error, Result };
use crate::meta::option::TextureOption;
use crate::runtime_meta::pack_version_specifier::PackVersionSpecifierRuntimeMeta;
use crate::runtime_meta::{ Message, MessageSeverity };
use crate::runtime_meta::version;
use crate::util::RON;
use std::collections::HashMap;
use super::META_NAME;
use tokio::fs;

#[derive(Debug)]
pub struct WithoutMCVersion(InnerWithoutMCVersion);

#[derive(Debug)]
pub struct InnerWithoutMCVersion {
	pub path: String,
	pub shortpath: String,
	pub name: String,
	pub description: String,
	pub versions: HashMap<String, version::WithoutMCVersion, RandomState>,
	pub messages: Vec<Message>
}

#[derive(Debug)]
pub enum WithMCVersion {
	Available(Available),
	Unavailable(Unavailable)
}

#[derive(Debug)]
pub struct Available(InnerAvailable);
#[derive(Debug)]
pub struct Unavailable(InnerUnavailable);


#[derive(Debug)]
pub struct InnerAvailable {
	pub path: String,
	pub shortpath: String,
	pub name: String,
	pub description: String,
	pub available_version: version::Available,
	pub unavailable_versions: HashMap<String, version::Unavailable, RandomState>,
	pub messages: Vec<Message>
}

#[derive(Debug)]
pub struct InnerUnavailable {
	pub path: String,
	pub shortpath: String,
	pub name: String,
	pub description: String,
	pub versions: HashMap<String, version::Unavailable, RandomState>,
	pub messages: Vec<Message>
}

crate::impl_deref!(WithoutMCVersion, target InnerWithoutMCVersion);
crate::impl_deref!(Available, target InnerAvailable);
crate::impl_deref!(Unavailable, target InnerUnavailable);

impl WithoutMCVersion {
	pub async fn new(path: &str) -> Result<Self> {
		let mut messages = vec![];
		let manifest_path = format!("{path}/{META_NAME}");

		let manifest_file_meta = fs::metadata(&manifest_path).await
			.map_err(|e| Error::ManifestDoesNotExist { path: manifest_path.clone(), source: e })?;
		if !manifest_file_meta.is_file() {
			return Err(Error::ManifestIsNotFile { path: manifest_path })
		}

		let file = fs::read_to_string(&manifest_path).await
			.map_err(|e| Error::IOError { source: e })?;
		let option = RON.from_str::<TextureOption>(&file)
			.map_err(|e| Error::ParseErrorRonSpannedError {
				path: manifest_path,
				source: e
			})?;

		struct Destructure {
			name: String,
			description: String
		}

		let Destructure { name, description } = match option {
			TextureOption::V1 { name, description } => {
				Destructure {
					name,
					description: description.unwrap_or_else(|| "description not provided".into())
				}
			}
		};

		let mut versions = HashMap::<String, version::WithoutMCVersion, RandomState>::new();

		let mut dir_contents = fs::read_dir(&path).await
			.map_err(|e| Error::IOError { source: e })?;
		while let Some(dir_entry) = dir_contents.next_entry().await.map_err(|e| Error::IOError { source: e })? {
			let dir_entry_path = dir_entry.path();
			let dir_entry_path = dir_entry_path.to_str()
				.expect("invalid unicode paths unsupported");

			if dir_entry_path.ends_with(META_NAME) { continue }

			let dir_entry_metadata = fs::metadata(&dir_entry_path).await
				.map_err(|e| Error::IOError { source: e })?;
			if !dir_entry_metadata.is_dir() {
				messages.push(Message {
					message: format!("item in an option dir is not a dir (potential version) or the manifest file: {dir_entry_path}"),
					severity: MessageSeverity::Info
				});
				continue
			}

			match version::WithoutMCVersion::new(dir_entry_path).await {
				Ok(version) => { versions.insert(version.shortpath.clone(), version); }
				Err(e) => { messages.push(e.to_message()) }
			}
		}

		let shortpath = std::path::Path::new(path)
			.file_name()
			.unwrap()
			.to_str()
			.unwrap()
			.into();

		if versions.is_empty() {
			messages.push(Error::UnavailableInfo {
				thing: format!("Option {shortpath}"),
				reason: "no versions are available".into()
			}.to_message());
		}

		Ok(Self(InnerWithoutMCVersion {
			path: path.into(),
			shortpath,
			name,
			description,
			versions,
			messages
		}))
	}
}

impl WithMCVersion {
	pub async fn from(
		option_without_mc_version: &WithoutMCVersion,
		mc_version: PackVersionSpecifierRuntimeMeta
	) -> Result<Self> {
		if option_without_mc_version.versions.is_empty() {
			return Ok(WithMCVersion::Unavailable(Unavailable(InnerUnavailable {
				path: option_without_mc_version.path.clone(),
				shortpath: option_without_mc_version.shortpath.clone(),
				name: option_without_mc_version.name.clone(),
				description: option_without_mc_version.description.clone(),
				versions: HashMapExt::new(),
				messages: option_without_mc_version.messages.clone()
			})))
		}

		let mut messages = option_without_mc_version.messages.clone();
		let mut available = HashMap::<String, version::Available, RandomState>::new();
		let mut unavailable = HashMap::<String, version::Unavailable, RandomState>::new();

		for (shortpath, version) in option_without_mc_version.versions.iter() {
			use version::WithMCVersion::*;
			match version::WithMCVersion::from(version, mc_version.clone()).await {
				Ok(version) => match version {
					Available(version) => { available.insert(shortpath.clone(), version); }
					Unavailable(version) => { unavailable.insert(shortpath.clone(), version); }
				}
				Err(e) => { messages.push(e.to_message()) }
			}
		}

		if available.is_empty() {
			messages.push(Error::UnavailableInfo {
				thing: format!("Option {}", option_without_mc_version.shortpath),
				reason: "no compatible versions are available".into()
			}.to_message());

			return Ok(Self::Unavailable(Unavailable(InnerUnavailable {
				path: option_without_mc_version.path.clone(),
				shortpath: option_without_mc_version.shortpath.clone(),
				name: option_without_mc_version.name.clone(),
				description: option_without_mc_version.description.clone(),
				versions: unavailable,
				messages
			})))
		}

		if available.len() > 1 {
			return Err(Error::MultipleAvailableVersions {
				available_versions_shortnames_formatted: {
					available.iter()
						.map::<&str, _>(|v| &v.1.shortpath)
						.collect::<Vec<_>>()
						.join(", ")
				}
			})
		}

		Ok(Self::Available(Available(InnerAvailable {
			path: option_without_mc_version.path.clone(),
			shortpath: option_without_mc_version.shortpath.clone(),
			name: option_without_mc_version.name.clone(),
			description: option_without_mc_version.description.clone(),
			available_version: available.into_iter().next().unwrap().1,
			unavailable_versions: unavailable,
			messages
		})))
	}
}
