use ::mc_versions::MCVersionRef;
use crate::util::fs;
use crate::util::path_builder3::WithOptionID;
use crate::provider2::{ self, ProviderRuntime };
use super::error::*;
use super::{ meta, nr };
use ::hashbrown::HashMap;
use ::serde::Serialize;

pub struct OptionRuntime {
	name: nr::Name,
	description: nr::Description,
	id: nr::ID,
	providers: nr::Providers
}

impl OptionRuntime {
	pub(crate) async fn new(p: &WithOptionID<'_>) -> Result<Option<Self>> {
		let dir = p.option_dir_silent_fail().await?;
		let meta_path = p.option_manifest_silent_fail().await?;
		let meta_file = fs::read_to_string2(meta_path).await?;
		let meta::OptionUnversioned {
			name,
			description
		} = meta::deserialise_option(&meta_file)?;

		let name = name.transmute_nom();
		let description = description.transmute_nom();
		let id = nr::ID::new(p.option_id_ref().into());

		let providers = read_providers(p).await?;

		Ok(Some(Self {
			name,
			description,
			id,
			providers
		}))
	}
}

async fn read_providers(p: &WithOptionID<'_>) -> Result<nr::Providers> {
	let version_entries_dir = p.provider_entries_dir_checked().await?;
	let mut versions_nom = nr::Providers::default();
	let versions = versions_nom.mut_inner();
	let mut read_dir = fs::read_dir2(version_entries_dir).await?;

	while let Some(file) = read_dir.next().await? {
		let file_name = file.file_name();
		let p = p.clone().with_provider_id_osstr(&file_name)?;

		// TODO
		if let Some(v) = ProviderRuntime::new(&p).await? {
			let id = provider2::nr::ID::new(p.provider_id_ref().into());
			versions.insert(id, v);
		}
	}

	Ok(versions_nom)
}

#[derive(Serialize)]
pub struct FrontendData<'h> {
	name: &'h nr::Name,
	description: &'h nr::Description,
	id: &'h nr::ID,
	available_providers: HashMap<&'h str, provider2::FrontendData<'h>>
}

impl<'h> FrontendData<'h> {
	pub fn new(option: &'h OptionRuntime, mc_version: MCVersionRef) -> Self {
		let name = &option.name;
		let description = &option.description;
		let id = &option.id;
		let available_providers = option.providers.ref_inner()
			.iter()
			.filter_map(|(id, p)| {
				provider2::FrontendData::new(p, mc_version)
					.map(|p| (&**id.ref_inner(), p))
			})
			.collect();

		Self { name, description, id, available_providers }
	}
}
