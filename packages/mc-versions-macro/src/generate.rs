#![allow(dead_code)]

use ::proc_macro2::TokenStream;
use ::quote::quote;
use ::serde::Deserialize;

const VERSION_MANIFEST_V2: &str = include_str!("./version_manifest_v2.json");
const VERSION_VALIDATION: &str = include_str!("./version_validation.txt");

#[derive(Deserialize)]
struct Manifest {
	latest: Latest,
	versions: Vec<Version>
}

#[derive(Deserialize)]
struct Latest {
	release: String,
	snapshot: String
}

#[derive(Clone, Deserialize)]
struct Version {
	id: String,
	r#type: String,
	url: String,
	time: String,
	#[serde(rename = "releaseTime")]
	release_time: String,
	sha1: String,
	#[serde(rename = "complianceLevel")]
	compliance_level: u8
}

/// the same one, copy pasted, as the one generated by the proc macro
#[derive(Deserialize)]
pub enum PackFormat {
	Verified(u8),
	Unverified(u8),
	Unknown,
	None
}

pub(crate) fn inject_generated_mc_versions(input: TokenStream) -> TokenStream {
	match inject_generated_mc_versions_inner(input) {
		Ok(t) | Err(t) => { t }
	}
}

fn inject_generated_mc_versions_inner(_: TokenStream) -> Result<TokenStream, TokenStream> {
	let manifest = ::serde_json::from_str::<Manifest>(VERSION_MANIFEST_V2)
		.map_err(|err| {
			let message = format!("parsing manifest had an error: {err}");
			quote! {
				compile_error!(#message);
			}
		})?;
	let version_validation = parse_version_validation(VERSION_VALIDATION)
		.map_err(|err| {
			quote! {
				compile_error!(#err);
			}
		})?;

	let Manifest { latest, versions } = manifest;

	let mut versions = versions.into_iter()
		.map(|v| {
			let parse = ::chrono::DateTime::parse_from_rfc3339;

			let release_time = parse(&v.release_time).unwrap();
			// in the extraordinary case of 1.6.3 and 13w37b, they have identical
			// release times, but their `time` field, whatever that is, isn't, so doing this
			// should mean sorting is *fully* stable, no matter the input order, as long as
			// the data itself doesn't change of course
			let time = parse(&v.time).unwrap();

			((release_time, time), v)
		})
		.collect::<Vec<_>>();

	versions.sort_by_key(|v| v.0);
	let mut versions = versions.into_iter()
		.zip(1..)
		.map(|((d, v), n)| (d, v, n))
		.collect::<Vec<_>>();
	versions.reverse();

	let mut max_version = 0u8;
	let versions = versions.into_iter()
		.map(|(_, v, n)| (gen_release(&v, n, &version_validation, &mut max_version), v))
		.collect::<Vec<_>>();

	let release = versions.iter()
		.find(|(_, v)| v.id == latest.release)
		.map(|(ts, _)| ts);
	let snapshot = versions.iter()
		.find(|(_, v)| v.id == latest.snapshot)
		.map(|(ts, _)| ts);

	let (release, snapshot) = match (release, snapshot) {
		(Some(release), Some(snapshot)) => {
			Ok((release, snapshot))
		}
		(Some(_), None) => { Err(quote! {
			compile_error!("manifest is invalid: latest snapshot does not have corresponding entry in versions");
		}) }
		(None, Some(_)) => { Err(quote! {
			compile_error!("manifest is invalid: latest release does not have corresponding entry in versions");
		}) }
		(None, None) => { Err(quote! {
			compile_error!("manifest is invalid: both latest release and snapshot do not have corresponding entries in versions");
		}) }
	}?;

	let versions = versions.iter().map(|(ts, _)| ts);

	Ok(quote! {
		pub const LATEST_RELEASE: MCVersion = #release;
		pub const LATEST_SNAPSHOT: MCVersion = #snapshot;

		pub const MAX_VERSION: u8 = #max_version;

		pub const MC_VERSIONS: &[MCVersion] = &[
			#( #versions ),*
		];
	})
}

fn parse_version_validation(file: &str) -> Result<Vec<(String, PackFormat)>, String> {
	let invalid_line_err = |l_no| format!("line #{l_no} in version_validation.txt is invalid");

	file
		.split('\n')
		.zip(1usize..)
		.map(|(l, i)| (l.trim(), i))
		.filter(|(l, _)| !l.is_empty())
		.map(|(l, i)| {
			l.split_once(' ')
				.map(|(pv, mcv)| (pv.trim(), mcv.trim()))
				.ok_or_else(|| invalid_line_err(i))
				.and_then(|(pv, mcv)| {
					let pv = if let Some(pv) = pv.strip_prefix("verified=") {
						pv.parse()
							.map(PackFormat::Verified)
							.map_err(|_| invalid_line_err(i))
					} else if let Some(pv) = pv.strip_prefix("unverified=") {
						pv.parse()
							.map(PackFormat::Unverified)
							.map_err(|_| invalid_line_err(i))
					} else if pv == "none" {
						Ok(PackFormat::None)
					} else if pv == "unknown" {
						Ok(PackFormat::Unknown)
					} else {
						Err(invalid_line_err(i))
					};

					pv.map(|pv| (mcv.into(), pv))
				})
		})
		.collect()
}

fn gen_release(
	Version { id: name, r#type, .. }: &Version,
	n: usize,
	version_validation: &[(String, PackFormat)],
	max_version: &mut u8
) -> TokenStream {
	let release_type = gen_release_type(r#type);
	let pack_format = gen_pack_format(name, version_validation, max_version);

	quote! {
		MCVersion {
			inner: Inner {
				name: #name,
				release_type: #release_type,
				pack_format: #pack_format,
				n: #n
			}
		}
	}
}

fn gen_release_type(release_type: &str) -> TokenStream {
	match release_type {
		"snapshot" => { quote! { ReleaseType::Snapshot } }
		"release" => { quote! { ReleaseType::Release } }
		"old_beta" => { quote! { ReleaseType::OldBeta } }
		"old_alpha" => { quote! { ReleaseType::OldAlpha } }
		t => { unreachable!("unexpectedly got \"{t}\" for a release type") }
	}
}

fn gen_pack_format(
	name: &str,
	version_validation: &[(String, PackFormat)],
	max_version: &mut u8
) -> TokenStream {
	let validated = version_validation.iter().find(|v| v.0 == name);
	if let Some((_, format)) = validated {
		match format {
			PackFormat::Verified(v) => {
				*max_version = u8::max(*max_version, *v);
				quote! { PackFormat::Verified(#v) }
			}
			PackFormat::Unverified(v) => {
				*max_version = u8::max(*max_version, *v);
				quote! { PackFormat::Unverified(#v) }
			}
			PackFormat::Unknown => { quote! { PackFormat::Unknown } }
			PackFormat::None => { quote! { PackFormat::None } }
		}
	} else {
		quote! { PackFormat::Unknown }
	}
}
