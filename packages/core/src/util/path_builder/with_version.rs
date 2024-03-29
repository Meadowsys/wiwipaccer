use crate::nom as n;
use super::super::error::*;
use super::super::fs;
use super::WithOption;
use ::camino::Utf8PathBuf;
use ::std::ops::Deref;

const VERSION_META_FILENAME: &str = "version.wiwimeta";

pub struct WithVersion<'h> {
	pub(super) prev: WithOption<'h>,
	pub(super) version_id: &'h str
}

impl<'h> WithVersion<'h> {
	#[inline]
	pub(super) fn _version_dir(&self) -> Utf8PathBuf {
		let mut path = self._option_dir();
		path.push(self.version_id);
		path
	}

	#[inline]
	pub async fn version_dir(&self) -> Result<n::global::VersionDirPath> {
		self.version_dir2().await.map(n::global::VersionDirPath::new)
	}

	#[inline]
	pub async fn version_dir2(&self) -> Result<String> {
		let path = unsafe { self.version_dir_unchecked2() };
		let res = fs::is_dir2(path.clone()).await?;

		if res {
			Ok(path)
		} else {
			let path_name = "version dir".into();
			Err(Error::PathIsNotDir { path, path_name })
		}
	}

	#[inline]
	pub unsafe fn version_dir_unchecked(&self) -> n::global::VersionDirPath {
		n::global::VersionDirPath::new(self.version_dir_unchecked2())
	}

	#[inline]
	pub unsafe fn version_dir_unchecked2(&self) -> String {
		self._version_dir().into_string()
	}

	#[inline]
	pub async fn version_manifest(&self) -> Result<n::global::VersionManifestPath> {
		self.version_manifest2().await.map(n::global::VersionManifestPath::new)
	}

	#[inline]
	pub async fn version_manifest2(&self) -> Result<String> {
		let path = unsafe { self.version_manifest_unchecked2() };
		let res = fs::is_file2(path.clone()).await?;

		if res {
			Ok(path)
		} else {
			let path_name = "version manifest".into();
			Err(Error::PathIsNotDir { path, path_name })
		}
	}

	#[inline]
	pub unsafe fn version_manifest_unchecked(&self) -> n::global::VersionManifestPath {
		n::global::VersionManifestPath::new(self.version_manifest_unchecked2())
	}

	#[inline]
	pub unsafe fn version_manifest_unchecked2(&self) -> String {
		let mut path = self._version_dir();
		path.push(VERSION_META_FILENAME);
		path.into_string()
	}
}

impl<'h> Deref for WithVersion<'h> {
	type Target = WithOption<'h>;

	#[inline]
	fn deref(&self) -> &Self::Target {
		&self.prev
	}
}
