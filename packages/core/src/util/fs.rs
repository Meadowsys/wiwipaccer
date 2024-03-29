#![deprecated]

use crate::nom as n;
use super::error::*;
use ::std::fs;
use ::std::io::Read as _;

#[inline]
pub async fn metadata(path: n::global::Path) -> Result<fs::Metadata> {
	metadata2(path.into_inner()).await
}

#[inline]
pub async fn metadata2(path: String) -> Result<fs::Metadata> {
	let f = || fs::metadata(path)
		.map_err(Error::FSError);
	spawn_blocking(f).await
}

#[inline]
pub async fn is_dir(path: n::global::Path) -> Result<bool> {
	is_dir2(path.into_inner()).await
}

#[inline]
pub async fn is_dir2(path: String) -> Result<bool> {
	Ok(metadata2(path).await?.is_dir())
}

#[inline]
pub async fn is_file(path: n::global::Path) -> Result<bool> {
	is_file2(path.into_inner()).await
}

#[inline]
pub async fn is_file2(path: String) -> Result<bool> {
	Ok(metadata2(path).await?.is_file())
}

#[inline]
pub async fn read_to_string(path: n::global::FilePath) -> Result<String> {
	read_to_string2(path.into_inner()).await
}

// TODO: can probably be optimised (one less meta call?) if rewritten by hand?
pub async fn read_to_string2(path: String) -> Result<String> {
	let f = || fs::read(path)
		.map_err(Error::FSError);
	let bytes = spawn_blocking(f).await?;

	match std::str::from_utf8(&bytes) {
		Ok(_) => { Ok(unsafe { String::from_utf8_unchecked(bytes) }) }
		Err(source) => { Err(Error::Utf8Error { source, bytes }) }
	}
}

#[inline]
pub async fn read_dir(path: n::global::DirPath) -> Result<ReadDir> {
	read_dir2(path.into_inner()).await
}

#[inline]
pub async fn read_dir2(path: String) -> Result<ReadDir> {
	tokio::fs::read_dir(path)
		.await
		.map(ReadDir)
		.map_err(Error::FSError)
}

#[inline]
async fn spawn_blocking<F, T>(f: F) -> Result<T>
where
	F: FnOnce() -> Result<T> + Send + 'static,
	T: Send + 'static
{
	match ::tokio::task::spawn_blocking(f).await {
		Ok(r) => { r }
		Err(e) => { Err(Error::BackgroundTaskFailed(e)) }
	}
}

#[repr(transparent)]
pub struct ReadDir(tokio::fs::ReadDir);

impl ReadDir {
	#[inline]
	pub async fn next(&mut self) -> Result<Option<tokio::fs::DirEntry>> {
		self.0.next_entry()
			.await
			.map_err(Error::FSError)
	}
}
