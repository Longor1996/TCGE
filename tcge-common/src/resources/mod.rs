pub use std::io::Read;

pub mod location;
pub use location::ResourceLocation;

pub mod providers;
pub use providers::*;

pub fn new() -> Resources {
	Resources::new()
}

pub enum ResourceError {
	Io(std::io::Error),
	HasNil(usize),
	NotFound,
	Unknown,
}

pub struct Resources {
	providers: Vec<Box<dyn ResourceProvider>>
}

impl Resources {
	pub fn new() -> Self {
		let mut new = Resources {providers: vec![]};
		
		let filesystem = FilesystemProvider::from_exe_path()
			.ok().expect("Could not open filesystem provider.");
		new.register_provider_by_type(filesystem);
		
		new
	}
	
	pub fn register_provider_by_type<P: 'static + ResourceProvider>(&mut self, provider: P) {
		self.providers.push(Box::new(provider));
	}
	
	pub fn register_provider(&mut self, provider: Box<dyn ResourceProvider>) {
		self.providers.push(provider);
	}
}

impl ResourceProvider for Resources {
	fn get_internal_name(&self) -> &str {
		"RootSet"
	}
	
	fn res_list(&self) -> Result<Box<dyn Iterator<Item = String>>, ResourceError> {
		let mut iter: Box<dyn Iterator<Item = String>> = Box::new(std::iter::empty());
		
		for provider in self.providers.iter() {
			let sub_iter = match provider.res_list() {
				Err(err) => return Err(err),
				Ok(iter) => iter,
			};
			
			iter = Box::new(iter.chain(sub_iter));
		}
		
		Ok(iter)
	}
	
	fn res_as_stream(&self, location: &ResourceLocation) -> Result<Box<dyn Read>, ResourceError> {
		debug!("Attempting to find resource: {}", location);
		
		for provider in self.providers.iter() {
			match provider.res_as_stream(location) {
				Err(_) => continue,
				Ok(stream) => {
					return Ok(stream)
				}
			}
		}
		
		error!("Failed to find resource: {}", location);
		Err(ResourceError::NotFound)
	}
}

pub fn get_exe_path() -> Result<std::path::PathBuf, std::io::Error> {
	let exe_file_name = ::std::env::current_exe()?;
	
	let exe_path = exe_file_name.parent()
		.ok_or(std::io::Error::from(std::io::ErrorKind::NotFound))?;
	
	Ok(exe_path.into())
}