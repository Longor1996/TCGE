use super::Gl;
use std::rc::Rc;

pub type BufferObjectRef = Rc<BufferObject>;

#[allow(dead_code)]
#[derive(Clone)]
pub struct BufferObject {
	pub id: gl::types::GLuint,
	pub target: gl::types::GLenum,
	pub items: usize,
	pub bytes: usize,
}

impl BufferObject {
	
	pub fn to_ref(self) -> BufferObjectRef {
		Rc::new(self)
	}
	
	/// Wraps GenBuffers & BufferData.
	///
	/// See: https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/glBufferData.xhtml
	#[allow(dead_code)]
	pub fn buffer_data<Element>(gl: &Gl, target: gl::types::GLenum, usage: gl::types::GLenum, elements: &[Element]) -> Self {
		let mut id = 0;
		let items = elements.len();
		let bytes = items * std::mem::size_of::<Element>();
		
		unsafe {
			gl.GenBuffers(1, &mut id);
			gl.BindBuffer(target, id);
			gl.BufferData(
				target,
				bytes as gl::types::GLsizeiptr,
				elements.as_ptr() as *const gl::types::GLvoid,
				usage
			);
			gl.BindBuffer(target, 0);
		}
		
		Self {
			id,
			target,
			items,
			bytes,
		}
	}
	
	/// Wraps GenBuffers & BufferData without upload.
	///
	/// See: https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/glBufferData.xhtml
	#[allow(dead_code)]
	pub fn buffer_data_empty<Element>(gl: &Gl, target: gl::types::GLenum, usage: gl::types::GLenum, elements: usize) -> Self {
		let mut id = 0;
		let items = elements;
		let bytes = items * std::mem::size_of::<Element>();
		
		unsafe {
			gl.GenBuffers(1, &mut id);
			gl.BindBuffer(target, id);
			gl.BufferData(
				target,
				bytes as gl::types::GLsizeiptr,
				std::ptr::null::<gl::types::GLvoid>(),
				usage
			);
			gl.BindBuffer(target, 0);
		}
		
		Self {
			id,
			target,
			items,
			bytes,
		}
	}
	/// Wraps GenBuffers & BufferStorage.
	///
	/// See: https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/glBufferData.xhtml
	#[allow(dead_code)]
	pub fn buffer_storage<Element>(gl: &Gl, target: gl::types::GLenum, flags: gl::types::GLbitfield, elements: &[Element]) -> Self {
		let mut id = 0;
		let items = elements.len();
		let bytes = items * std::mem::size_of::<Element>();
		
		unsafe {
			gl.GenBuffers(1, &mut id);
			gl.BindBuffer(target, id);
			gl.BufferStorage(
				target,
				bytes as gl::types::GLsizeiptr,
				elements.as_ptr() as *const gl::types::GLvoid,
				flags
			);
			gl.BindBuffer(target, 0);
		}
		
		Self {
			id,
			target,
			items,
			bytes,
		}
	}
	
	/// Wraps GenBuffers & BufferStorage without upload.
	///
	/// See: https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/glBufferData.xhtml
	#[allow(dead_code)]
	pub fn buffer_storage_empty<Element>(gl: &Gl, target: gl::types::GLenum, flags: gl::types::GLbitfield, elements: usize) -> Self {
		let mut id = 0;
		let items = elements;
		let bytes = items * std::mem::size_of::<Element>();
		
		unsafe {
			gl.GenBuffers(1, &mut id);
			gl.BindBuffer(target, id);
			gl.BufferStorage(
				target,
				bytes as gl::types::GLsizeiptr,
				std::ptr::null::<gl::types::GLvoid>(),
				flags
			);
			gl.BindBuffer(target, 0);
		}
		
		Self {
			id,
			target,
			items,
			bytes,
		}
	}
	
	pub fn buffer_mapped_upload<Element>(&self, gl: &Gl, elements: &[Element]) -> Result<(), String> {
		let element_size = std::mem::size_of::<Element>();
		let elements_len = elements.len();
		let elements_bytes = elements_len * element_size;
		
		if elements_bytes > self.bytes {
			return Err(format!(
				"Cannot upload data to buffer {}: Too many elements; {} > {}",
				self.id, elements_bytes, self.bytes
			));
		}
		
		unsafe {
			gl.BindBuffer(self.target, self.id);
			let handle = gl.MapBuffer(self.target, gl::WRITE_ONLY) as *mut Element;
			
			if handle.is_null() {
				return Err(format!(
					"Cannot upload data to buffer {}: OpenGL returned NIL-handle.",
					self.id
				));
			}
			
			elements.as_ptr().copy_to(handle, elements_bytes);
			
			gl.UnmapBuffer(self.target);
			gl.BindBuffer(self.target, 0);
		}
		
		Ok(())
	}
}
