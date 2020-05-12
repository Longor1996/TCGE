
use image::GenericImageView;
use common::resources;

pub struct TextureObjectBuilder {
	name: Option<String>,
	
	target: gl::types::GLenum,
	
	wrapping: gl::types::GLenum,
	
	mipmaps: bool,
	filter_min: gl::types::GLenum,
	filter_mag: gl::types::GLenum,
	anisotropy: bool,
	
	internal_format: gl::types::GLenum,
}

impl TextureObjectBuilder {
	#[allow(dead_code)]
	pub fn new() -> Self {
		Self {
			name: None,
			
			target: gl::TEXTURE_2D,
			
			wrapping: gl::REPEAT,
			
			mipmaps: true,
			filter_min: gl::LINEAR_MIPMAP_LINEAR,
			filter_mag: gl::LINEAR_MIPMAP_LINEAR,
			anisotropy: false,
			
			internal_format: gl::RGBA,
		}
	}
	
	#[allow(dead_code)]
	pub fn name(mut self, name: String) -> Self {
		self.name = Some(name);
		self
	}
	
	#[allow(dead_code)]
	pub fn wrapping(mut self, wrapping: gl::types::GLenum) -> Self {
		self.wrapping = wrapping;
		self
	}
	
	#[allow(dead_code)]
	pub fn mipmaps(mut self, mipmaps: bool) -> Self {
		self.mipmaps = mipmaps;
		self
	}
	
	#[allow(dead_code)]
	pub fn filter(mut self, min: gl::types::GLenum, mag: gl::types::GLenum) -> Self {
		self.filter_min = min;
		self.filter_mag = mag;
		self
	}
	
	#[allow(dead_code)]
	pub fn anisotropy(mut self, anisotropy: bool) -> Self {
		self.anisotropy = anisotropy;
		self
	}
	
	#[allow(dead_code)]
	pub fn internal_format(mut self, internal_format: gl::types::GLenum) -> Self {
		self.internal_format = internal_format;
		self
	}
	
	#[allow(dead_code)]
	pub fn build_from_res(
		self,
		gl: &super::Gl,
		res: &resources::Resources,
		location: &resources::ResourceLocation
	) -> Result<TextureObject, TextureError> {
		// load buffer
		use resources::ResourceProvider;
		let buffer = res.res_as_buffer(location)
			.map_err(TextureError::Resource)?;
		
		// parse buffer
		self.build_from_buffer(gl, &buffer)
	}
	
	#[allow(dead_code)]
	pub fn build_from_buffer(
		self,
		gl: &super::Gl,
		buffer: &[u8]
	) -> Result<TextureObject, TextureError> {
		// Parse image
		let image = image::load_from_memory(buffer)
			.map_err(TextureError::Image)?;
		
		self.build_from_dynamic_image(gl, &image)
	}
	
	pub fn build_from_dynamic_image(
		self,
		gl: &super::Gl,
		image: &image::DynamicImage,
	) -> Result<TextureObject, TextureError> {
		// Get size
		let image_size = image.dimensions();
		let width = image_size.0;
		let height = image_size.1;
		
		// Convert to RGBA
		// TODO: Maybe automate this one day?
		let image = image.to_rgba();
		
		let target = self.target;
		
		let mut id: gl::types::GLuint = 0;
		unsafe {
			gl.push_debug(&format!("Uploading texture: {}x{}", width, height));
			
			// create & bind
			gl.GenTextures(1, &mut id);
			gl.BindTexture(target, id);
			
			if let Some(name) = self.name {
				gl.label_object(
					gl::TEXTURE,
					id,
					&format!("Texture: {}", name)
				);
			}
			
			// wrapping
			gl.TexParameteri(target, gl::TEXTURE_WRAP_S, self.wrapping as gl::types::GLint);
			gl.TexParameteri(target, gl::TEXTURE_WRAP_T, self.wrapping as gl::types::GLint);
			
			// sampling
			gl.TexParameteri(target, gl::TEXTURE_MIN_FILTER, self.filter_min as gl::types::GLint);
			gl.TexParameteri(target, gl::TEXTURE_MAG_FILTER, self.filter_mag as gl::types::GLint);
			
			if self.anisotropy {
				// Attempt to enable anisotropic filtering...
				let mut aniso: f32 = 0.0;
				gl.GetFloatv(0x84FF, &mut aniso);
				if aniso != 0.0 {
					gl.TexParameterf(gl::TEXTURE_2D, 0x84FE, aniso);
				}
			}
			
			gl.TexImage2D(
				target,
				0,
				self.internal_format as gl::types::GLint,
				width as gl::types::GLsizei,
				height as gl::types::GLsizei,
				0,
				gl::RGBA,
				gl::UNSIGNED_BYTE,
				image.as_ptr() as *const std::ffi::c_void
			);
			
			while let Some(error) = gl.get_error() {
				error!("OpenGL error while uploading texture: {}", error);
			}
			
			if self.mipmaps {
				// Gen mipmaps
				gl.GenerateMipmap(target);
			}
			
			// unbind
			gl.BindTexture(target, 0);
			gl.pop_debug();
		}
		
		let tx = 1.0 / width as f32;
		let ty = 1.0 / height as f32;
		
		Ok(TextureObject {
			gl: gl.clone(),
			id,
			target,
			width,
			height,
			tx,
			ty,
		})
	}
}

pub struct TextureObject {
	gl: gl::Gl,
	id: gl::types::GLuint,
	target: gl::types::GLenum,
	
	width: u32,
	height: u32,
	tx: f32,
	ty: f32,
}

impl TextureObject {
	#[allow(dead_code)]
	pub fn set_used(&self) {
		unsafe {
			self.gl.BindTexture(self.target, self.id)
		}
	}
	
	#[allow(dead_code)]
	pub fn dimensions(&self) -> (u32, u32) {
		(self.width, self.height)
	}
	
	#[allow(dead_code)]
	pub fn width(&self) -> u32 {
		self.width
	}
	
	#[allow(dead_code)]
	pub fn height(&self) -> u32 {
		self.height
	}
	
	#[allow(dead_code)]
	pub fn get_uv_rect(&self, x: u32, y:u32, w: u32, h: u32) -> [f32;4] {
		[
			x as f32 * self.tx,
			y as f32 * self.ty,
			(x+w) as f32 * self.tx,
			(y+h) as f32 * self.ty
		]
	}
}

pub enum TextureError {
	Image(image::ImageError),
	Resource(resources::ResourceError),
}
