use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[repr(transparent)]
#[derive(Debug, Error)]
#[error(transparent)]
pub struct Error(pub(crate) ErrorInner);

#[derive(Debug, Error)]
pub(crate) enum ErrorInner {
	#[error("background task failed:\n{0}")]
	BackgroundTaskFailed(#[source] tokio::task::JoinError),

	#[error("FS error:\n{0}")]
	FSError(#[source] std::io::Error),

	#[error("error parsing ron:\n{0}")]
	RonError(#[from] ron::error::Error),

	#[error("error parsing ron:\n{0}")]
	RonSpannedError(#[from] ron::error::SpannedError),

	#[error("error parsing utf-8 text: {source}")]
	Utf8Error {
		source: std::str::Utf8Error,
		bytes: Vec<u8>
	}
}
