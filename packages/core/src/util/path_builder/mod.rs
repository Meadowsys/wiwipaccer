mod root;
mod with_texture;
mod with_option;
mod with_version;

use crate::nom as n;
pub use root::Root;
pub use with_texture::WithTexture;
pub use with_option::WithOption;
pub use with_version::WithVersion;

#[inline]
pub fn path_builder(root_dir: &n::global::RootDirPath) -> Root {
	Root { root_dir }
}