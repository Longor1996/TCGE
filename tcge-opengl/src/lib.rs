//! This library crate contains the (generated) OpenGL bindings used by the client.

/// The actual (automatically generated) OpenGL bindings.
pub mod bindings;

pub use bindings::*;
pub use bindings::Gl as InnerGl;

use std::rc::Rc;
use std::ffi::{CString};

/// Reference-counted struct of OpenGL function pointers.
#[derive(Clone)]
pub struct Gl {
	inner: Rc<bindings::Gl>,
	debug: bool
}

impl Gl {
	/// Loads the function-pointers for a new OpenGL context.
	pub fn load_with<F>(loadfn: F) -> Gl
		where F: FnMut(&'static str) -> *const types::GLvoid
	{
		Gl {
			inner: Rc::new(bindings::Gl::load_with(loadfn)),
			debug: false,
		}
	}
	
	pub fn debug(&mut self, func: bindings::types::GLDEBUGPROC, severity: bindings::types::GLenum) {
		self.debug = true;
		unsafe {
			self.Enable(bindings::DEBUG_OUTPUT);
			self.DebugMessageCallback(func, 0 as *const std::ffi::c_void);
			self.DebugMessageControl(
				bindings::DONT_CARE,
				bindings::DONT_CARE,
				severity,
				0,
				0 as *const bindings::types::GLuint,
				bindings::TRUE
			);
		}
	}
	
	pub fn flush(&self) {
		unsafe {
			self.Flush()
		}
	}
	
	pub fn label_object(&self, identifier: bindings::types::GLenum, name: bindings::types::GLuint, label: &str) {
		if ! self.debug {return}
		let obj_name = CString::new(label).expect("Could not convert name into C-String.");
		unsafe {
			self.ObjectLabel(
				identifier,
				name,
				label.len() as i32,
				obj_name.as_ptr()
			);
		}
	}
	
	pub fn push_debug(&self, message: &str) {
		if ! self.debug {return}
		let msg = CString::new(message).expect("Failed to convert to C-Str.");
		self.push_debug_raw(&msg, message.len());
	}
	
	pub fn push_debug_raw(&self, message: &CString, length: usize) {
		if ! self.debug {return}
		unsafe {
			self.PushDebugGroup(
				bindings::DEBUG_SOURCE_APPLICATION,
				0,
				length as i32,
				message.as_ptr()
			);
		}
	}
	
	pub fn pop_debug(&self) {
		if ! self.debug {return}
		unsafe {
			self.PopDebugGroup();
		}
	}
	
	pub fn scope_debug<F, V>(&self, message: &str, func: F) -> V
		where F: FnOnce() -> V {
		self.push_debug(message);
		let v = func();
		self.pop_debug();
		v
	}
	
	pub fn get_error(&self) -> Option<&'static str> {
		if ! self.debug {return None}
		
		let error = unsafe {self.GetError()};
		
		if error == bindings::NO_ERROR {
			return None;
		}
		
		let error = match error {
			bindings::INVALID_ENUM      => "INVALID_ENUM",
			bindings::INVALID_VALUE     => "INVALID_VALUE",
			bindings::INVALID_OPERATION => "INVALID_OPERATION",
			bindings::INVALID_FRAMEBUFFER_OPERATION => "INVALID_FRAMEBUFFER_OPERATION",
			bindings::OUT_OF_MEMORY   => "OUT_OF_MEMORY",
			bindings::STACK_UNDERFLOW => "STACK_UNDERFLOW",
			bindings::STACK_OVERFLOW  => "STACK_OVERFLOW",
			_ => "UNKNOWN",
		};
		
		Some(error)
	}
	
	pub fn debugging(&self) -> bool {
		self.debug
	}
}

use std::ops::Deref;
impl Deref for Gl {
	type Target = bindings::Gl;
	
	fn deref(&self) -> &bindings::Gl {
		&self.inner
	}
}
