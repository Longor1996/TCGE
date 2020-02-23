use super::ResourceLocation;
use super::ResourceError;
use std::path::PathBuf;
use std::io::Read;
use std::fs::File;


pub struct FilesystemProvider {
	root_path: PathBuf
}

impl FilesystemProvider {
	pub fn from_exe_path() -> Result<FilesystemProvider, ResourceError> {
		let exe_path = super::super::get_exe_path()
			.map_err(ResourceError::Io)?;
		let fs_path = exe_path.join("assets");
		
		info!("Created FilesystemProvider: {}", fs_path.to_str().unwrap_or("[ERROR]"));
		Ok(FilesystemProvider {
			root_path: fs_path
		})
	}
}

impl super::ResourceProvider for FilesystemProvider {
	fn get_internal_name(&self) -> &str {
		"Filesystem"
	}
	
	fn res_list(&self) -> Result<Box<dyn Iterator<Item = String>>, ResourceError> {
		let root = self.root_path.clone();
		let walker = walkdir::WalkDir::new(root.clone())
			.max_depth(5)
			.same_file_system(true)
		;
		
		// This is a nightmare.
		let walker_iter = walker.into_iter()
			// Step 1: Get all the accessible DirEntry's
			.map(|e| e.ok())
			.filter(|e| e.is_some())
			.map(|e| e.unwrap())
			// Step 2: Only files.
			.filter(|e| e.file_type().is_file())
			// Step 3: Get the path of the file.
			.map(|e| e.path().to_owned())
			// Step 4: Convert path to String.
			.map(move |e| {
				// TODO: There HAS to be a better way to do this...
				e.strip_prefix(&root).unwrap()
					.to_str().unwrap_or("")
					.to_string()
			})
			// Step 5: Ignore failed paths.
			.filter(|e| !e.is_empty())
		;
		
		Ok(Box::new(walker_iter))
	}
	
	fn res_as_stream(&self, location: &ResourceLocation) -> Result<Box<dyn Read>, ResourceError> {
		let mut path: PathBuf = self.root_path.clone();
		
		// Instead of passing the path directly...
		for part in location.inner.split('/') {
			path = path.join(part);
		}
		
		let file = File::open(path)
			.map_err(ResourceError::Io)?;
		
		Ok(Box::new(file))
	}
}
