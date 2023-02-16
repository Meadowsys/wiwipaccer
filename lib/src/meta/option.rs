//! Option for a texture.

use serde::{ Deserialize, Serialize };

#[derive(Deserialize, Serialize)]
pub enum TextureOption {
	V1 {
		/// name of option
		name: String,
		/// description of option
		description: Option<String>
	}
}
