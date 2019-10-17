use super::ResourceLocation;
use super::ResourceError;
use std::io::Read;

pub type Includes = Vec<(&'static str, &'static [u8])>;

pub struct IncludeProvider {
	includes: Includes
}

impl IncludeProvider {
	pub fn new(includes: Includes) -> Self {
		info!("Created IncludeProvider: {} Items", includes.len());
		
		Self {
			includes
		}
	}
}

impl super::ResourceProvider for IncludeProvider {
	fn get_internal_name(&self) -> &str {
		"Includes"
	}
	
	fn res_list(&self) -> Result<Box<dyn Iterator<Item = String>>, ResourceError> {
		let iter: Vec<String> = self.includes.iter()
			.map(|(name, _)| (*name).to_string())
			.collect()
		;
		Ok(Box::new(iter.into_iter()))
	}
	
	fn res_as_stream(&self, location: &ResourceLocation) -> Result<Box<dyn Read>, ResourceError> {
		let location = &location.inner;
		let file = self.includes.iter()
			.find(|(name, _bytes)| name == location);
		
		match file {
			None => Err(ResourceError::NotFound),
			Some((_name, bytes)) => {
				let r = bytes.clone();
				let r = Box::new(r);
				Ok(r)
			}
		}
	}
}
