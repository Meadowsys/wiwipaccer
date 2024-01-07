use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[repr(transparent)]
#[derive(Debug, Error)]
#[error(transparent)]
pub struct Error(pub(crate) ErrorInner);

#[derive(Debug, Error)]
pub(crate) enum ErrorInner {
	#[error("non UTF-8 paths are not supported")]
	NonUtf8Path,

	#[error(transparent)]
	UtilError(#[from] crate::util::error::Error),

	#[error(transparent)]
	VersionError(#[from] crate::version::error::Error)
}

impl crate::util::IntoError for Error {
	type Inner = ErrorInner;
	#[inline]
	fn with_inner(inner: Self::Inner) -> Self {
		Error(inner)
	}
}
