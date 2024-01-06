use crate::nom as n;
use ::thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[repr(transparent)]
#[derive(Debug, Error)]
#[error(transparent)]
pub struct Error(pub(crate) ErrorInner);

#[derive(Debug, Error)]
pub(crate) enum ErrorInner {
	#[error(
		"dependencies not satisfied: {}",
		.0.iter()
			.map(|(id, req, v)| match v {
				Some(v) => { format!("{id} {req} ({v} available)") }
				None => { format!("{id} {req}") }
			})
			.collect::<Vec<_>>()
			.join(", ")
	)]
	DepsNotSatisfied(Vec<(n::pack::ID, ::semver::VersionReq, Option<::semver::Version>)>),

	#[error("non UTF-8 paths are not supported")]
	NonUtf8Path,

	#[error("error parsing semver:\n{0}")]
	SemverParseError(#[from] semver::Error),

	#[error(transparent)]
	TextureError(#[from] crate::texture::error::Error),

	#[error(transparent)]
	UtilError(#[from] crate::util::error::Error)
}

impl crate::util::IntoError for Error {
	type Inner = ErrorInner;
	#[inline]
	fn with_inner(inner: Self::Inner) -> Self {
		Error(inner)
	}
}
