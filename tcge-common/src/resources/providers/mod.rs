use super::ResourceLocation;
use super::ResourceError;
use std::io::Read;
use std::ffi::CString;

pub mod filesystem;
pub use filesystem::*;

pub mod include;
pub use include::*;

pub mod archive;
pub use archive::*;

pub trait ResourceProvider {
	fn get_internal_name(&self) -> &str;
	
	fn res_list(&self) -> Result<Box<dyn Iterator<Item = String>>, ResourceError>;
	
	fn res_as_stream(&self, location: &ResourceLocation) -> Result<Box<dyn Read>, ResourceError>;
	
	fn res_as_buffer(&self, location: &ResourceLocation) -> Result<Vec<u8>, ResourceError> {
		// If the extra semicolon is removed, the code does not compile, so it gets to stay.
		#![allow(redundant_semicolons)]
		
		let mut stream = self.res_as_stream(location)?;
		let mut buf = Vec::<u8>::new();
		
		stream.read_to_end(&mut buf)
			.map_err(ResourceError::Io)
			?;;
		
		Ok(buf)
	}
	
	fn res_as_string(&self, location: &ResourceLocation) -> Result<String, ResourceError> {
		let mut stream = self.res_as_stream(location)?;
		let mut buf = String::new();
		
		stream.read_to_string(&mut buf)
			.map_err(ResourceError::Io)
			?;
		
		Ok(buf)
	}
	
	fn res_as_cstring(&self, location: &ResourceLocation) -> Result<CString, ResourceError> {
		let buf = self.res_as_buffer(location)?;
		
		if let Some(position) = buf.iter().position(|i| *i == 0) {
			return Err(ResourceError::HasNil(position));
		}
		
		Ok(unsafe { CString::from_vec_unchecked(buf) })
	}
}
