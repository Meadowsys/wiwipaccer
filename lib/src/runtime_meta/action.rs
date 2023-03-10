#[derive(Debug)]
pub enum Action {
	CopyFile {
		/// this must be absolute
		from: String,
		/// this is relative to the base of the built resource pack
		to: String
	},
	WriteBytes {
		data: Vec<u8>,
		/// is is relative to the base of the built resource pack
		path: String,
		/// vec of paths of files that caused the compilation of this entry,
		/// the "source" if you will
		src_files: Vec<String>
	}
}
