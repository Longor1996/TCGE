//! A collection of often-used utilities for interfacing with OpenGL.

// TODO: Once CDML is implemented, rewrite loading to be more... dynamic.

use super::super::super::resources;
use std::ffi::{CString, CStr};
use super::cgmath::prelude::*;

#[derive(Debug, Fail)]
pub enum Error {
	#[fail(display = "Failed to load resource {}", name)]
	ResourceLoad { name: String, #[cause] inner: resources::ResError },
	
	#[fail(display = "Can not determine shader type for resource {}", name)]
	CanNotDetermineShaderTypeForResource { name: String },
	
	#[fail(display = "Failed to compile shader {}: {}", name, message)]
	CompileError { name: String, message: String },
	
	#[fail(display = "Failed to link program {}: {}", name, message)]
	LinkError { name: String, message: String },
	
	#[fail(display = "Failed to parse image {}", name)]
	ImageParse { name: String, #[cause] inner: image::ImageError },
	
	#[fail(display = "Failed to parse value {}", name)]
	ValueParse { name: String },
}

pub struct Program {
	name: String,
	id: gl::types::GLuint,
}

impl Program {
	pub fn from_res(res: &resources::Resources, name: &str) -> Result<Program, Error> {
		const POSSIBLE_EXT: [&str; 2] = [
			".vert",
			".frag",
		];
		
		let resource_names = POSSIBLE_EXT.iter()
			.map(|file_extension| format!("{}{}", name, file_extension))
			.collect::<Vec<String>>();
		
		info!("Loading program shaders: {}", name);
		let shaders = resource_names.iter()
			.map(|resource_name| {
				Shader::from_res(res, resource_name)
			})
			.collect::<Result<Vec<Shader>, Error>>()?;
		
		debug!("Compiling program: {}", name);
		Program::from_shaders(name, &shaders[..])
			.map_err(|message| Error::LinkError { name: name.into(), message })
	}
	
	pub fn from_shaders(name: &str, shaders: &[Shader]) -> Result<Program, String> {
		let program_id = unsafe { gl::CreateProgram() };
		
		for shader in shaders {
			unsafe {
				gl::AttachShader(program_id, shader.id());
				
				gl_label_object(
					gl::SHADER,
					shader.id(),
					&format!("{}{}", name, shader.kind_as_str())
				);
			}
		}
		
		unsafe { gl::LinkProgram(program_id); }
		
		let mut success: gl::types::GLint = 1;
		unsafe {
			gl::GetProgramiv(program_id, gl::LINK_STATUS, &mut success);
		}
		
		if success == 0 {
			let mut len: gl::types::GLint = 0;
			unsafe {
				gl::GetProgramiv(program_id, gl::INFO_LOG_LENGTH, &mut len);
			}
			
			let error = create_whitespace_cstring_with_len(len as usize);
			unsafe {
				gl::GetProgramInfoLog(
					program_id,
					len,
					std::ptr::null_mut(),
					error.as_ptr() as *mut gl::types::GLchar
				);
			}
			
			return Err(error.to_string_lossy().into_owned());
		}
		
		for shader in shaders {
			unsafe { gl::DetachShader(program_id, shader.id()); }
		}
		
		gl_label_object(gl::PROGRAM, program_id, name);
		
		Ok(Program { name: name.to_string(), id: program_id })
	}
	
	pub fn id(&self) -> gl::types::GLuint {
		self.id
	}
	
	pub fn set_used(&self) {
		unsafe {
			gl::UseProgram(self.id);
		}
	}
	
	pub fn uniform_location(&self, uniform_name: &str) -> i32 {
		let vstr = uniform_name.as_bytes().to_owned();
		let cstr = unsafe { CString::from_vec_unchecked(vstr) };
		unsafe {
			let loc = gl::GetUniformLocation(self.id, cstr.as_ptr());
			
			if loc == -1 {
				warn!("Uniform location for '{}' in '{}' is invalid.", uniform_name, self.name);
				return -1;
			}
			
			loc
		}
	}
	
	pub fn uniform_sampler(&self, uniform: i32, value: gl::types::GLuint) {
		if uniform == -1 {return}
		unsafe {
			gl::Uniform1i(uniform, value as i32)
		}
	}
	
	pub fn uniform_scalar(&self, uniform: i32, value: f32) {
		if uniform == -1 {return}
		unsafe {
			gl::Uniform1f(uniform, value)
		}
	}
	
	pub fn uniform_vector4(&self, uniform: i32, vector: cgmath::Vector4<f32>) {
		if uniform == -1 {return}
		unsafe {
			gl::Uniform4fv(uniform, 1, vector.as_ptr())
		}
	}
	
	pub fn uniform_matrix4(&self, uniform: i32, matrix: cgmath::Matrix4<f32>) {
		if uniform == -1 {return}
		unsafe {
			gl::UniformMatrix4fv(uniform, 1, gl::FALSE, matrix.as_ptr())
		}
	}
}

impl Drop for Program {
	fn drop(&mut self) {
		unsafe {
			gl::DeleteProgram(self.id);
		}
	}
}

pub struct Shader {
	id: gl::types::GLuint,
	kind: gl::types::GLenum,
}

impl Shader {
	pub fn id(&self) -> gl::types::GLuint {
		self.id
	}
	
	pub fn kind_as_str(&self) -> &str {
		match self.kind {
			gl::VERTEX_SHADER   => ".vert",
			gl::FRAGMENT_SHADER => ".frag",
			gl::GEOMETRY_SHADER => ".geom",
			_ => "UNKNOWN"
		}
	}
	
	pub fn from_res(
		res: &resources::Resources,
		name: &str
	) -> Result<Shader, Error>{
		const POSSIBLE_EXT: [(&str, gl::types::GLenum); 3] = [
			(".vert", gl::VERTEX_SHADER),
			(".frag", gl::FRAGMENT_SHADER),
			(".geom", gl::GEOMETRY_SHADER),
		];
		
		let shader_kind = POSSIBLE_EXT.iter()
			.find(|&&(file_extension, _)| {
				name.ends_with(file_extension)
			})
			.map(|&(_, kind)| kind)
			.ok_or_else(|| Error::CanNotDetermineShaderTypeForResource { name: name.into() })?;
		
		debug!("Loading shader: {} . {}", name, shader_kind);
		let source = res.load_cstring(name)
			.map_err(|e| Error::ResourceLoad { name: name.into(), inner: e })?;
		
		// TODO: Allow shader-files to include other shader-files.
		
		debug!("Compiling shader: {}", name);
		Shader::from_source(&source, shader_kind)
			.map_err(|message| Error::CompileError { name: name.into(), message })
	}
	
	pub fn from_source(
		source: &CStr,
		kind: gl::types::GLenum
	) -> Result<Shader, String> {
		let id = shader_from_source(source, kind)?;
		Ok(Shader {id, kind})
	}
	
	pub fn from_vert_source(source: &CStr) -> Result<Shader, String> {
		Shader::from_source(source, gl::VERTEX_SHADER)
	}
	
	pub fn from_frag_source(source: &CStr) -> Result<Shader, String> {
		Shader::from_source(source, gl::FRAGMENT_SHADER)
	}
}

impl Drop for Shader {
	fn drop(&mut self) {
		unsafe {
			gl::DeleteShader(self.id);
		}
	}
}

fn shader_from_source(
	source: &CStr,
	kind: gl::types::GLuint
) -> Result<gl::types::GLuint, String> {
	let id = unsafe { gl::CreateShader(kind) };
	
	unsafe {
		gl::ShaderSource(id, 1, &source.as_ptr(), std::ptr::null());
		gl::CompileShader(id);
	}
	
	let mut success: gl::types::GLint = 1;
	unsafe {
		gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut success);
	}
	
	if success == 0 {
		let mut len: gl::types::GLint = 0;
		unsafe {
			gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut len);
		}
		
		let error = create_whitespace_cstring_with_len(len as usize);
		unsafe {
			gl::GetShaderInfoLog(
				id, len,
				std::ptr::null_mut(),
				error.as_ptr() as *mut gl::types::GLchar
			);
		}
		
		return Err(error.to_string_lossy().into_owned());
	}
	
	Ok(id)
}

pub fn create_whitespace_cstring_with_len(len: usize) -> CString {
	let mut buffer: Vec<u8> = Vec::with_capacity(len as usize + 1);
	buffer.extend([b' '].iter().cycle().take(len as usize));
	unsafe { CString::from_vec_unchecked(buffer) }
}

pub struct Texture {
	/// OpenGL Texture Object Handle
	pub id: gl::types::GLuint,
	
	/// Horizontal size of the texture in pixels.
	#[allow(unused)]
	width: u32,
	
	/// Vertical size of the texture in pixels.
	#[allow(unused)]
	height: u32,
	
	/// Horizontal size of a single pixel within the texture.
	tx: f32,
	
	/// Vertical size of a single pixel within the texture.
	ty: f32,
}

impl Texture {
	
	pub fn from_res(res: &resources::Resources, name: &str) -> Result<Texture, Error> {
		// TODO: The following is rather horrible code. Causes way too many copies.
		
		let buffer = res.load_buffer(name)
			.map_err(|e| Error::ResourceLoad { name: name.into(), inner: e })?;
		
		let image = image::load_from_memory(&buffer)
			.map_err(|e| Error::ImageParse { name: name.into(), inner: e })?;
		
		let image = image.as_rgba8()
			.expect("Failed to convert Image to RGBA8.");
		
		let image_size = image.dimensions();
		let image_width = image_size.0;
		let image_height = image_size.1;
		
		let mut handle: gl::types::GLuint = 0;
		unsafe {
			// preparing
			gl::GenTextures(1, &mut handle);
			gl::BindTexture(gl::TEXTURE_2D, handle);
			
			// wrapping
			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
			
			// sampling
			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as i32);
			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR_MIPMAP_LINEAR as i32);
			
			// uploading
			gl::TexImage2D(gl::TEXTURE_2D,
			               0, gl::RGBA as i32,
			               image_width as i32, image_height as i32,
			               0, gl::RGBA, gl::UNSIGNED_BYTE,
			               image.as_ptr() as *const std::ffi::c_void
			);
			
			// Gen mipmaps
			gl::GenerateMipmap(gl::TEXTURE_2D);
		}
		
		gl_label_object(
			gl::TEXTURE,
			handle,
			name
		);
		
		Ok(Texture{
			id: handle,
			width: image_width,
			height: image_height,
			tx: 1.0 / image_width as f32,
			ty: 1.0 / image_height as f32
		})
	}
	
	pub fn get_uv_rect(&self, x: u32, y:u32, w: u32, h: u32) -> [f32;4] {
		return [
			x as f32 * self.tx,
			y as f32 * self.ty,
			(x+w) as f32 * self.tx,
			(y+h) as f32 * self.ty
		]
	}
	
}

pub fn gl_label_object(identifier: gl::types::GLenum, name: gl::types::GLuint, label: &str) {
	let obj_name = CString::new(label).expect("Could not convert name into C-String.");
	unsafe {
		gl::ObjectLabel(
			identifier,
			name,
			label.len() as i32,
			obj_name.as_ptr()
		);
	}
}

pub fn gl_push_debug(message: &str) {
	let msg = CString::new(message).expect("Failed to convert to C-Str.");
	unsafe {
		gl::PushDebugGroup(
			gl::DEBUG_SOURCE_APPLICATION,
			42,
			message.len() as i32,
			msg.as_ptr()
		);
	}
}

pub fn gl_pop_debug() {
	unsafe {
		gl::PopDebugGroup();
	}
}