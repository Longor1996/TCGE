pub struct GlfwContext {
	pub title_fin:  String,
	pub title_dyn:  String,
	pub glfw:   glfw::Glfw,
	pub window: glfw::Window,
	pub events: std::sync::mpsc::Receiver<(f64, glfw::WindowEvent)>,
	pub gl:     gl::Gl,
	pub gl_info: GlInfo,
}



impl GlfwContext {
	pub fn new() -> Self {
		let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
		
		glfw.window_hint(glfw::WindowHint::Visible(false));
		glfw.window_hint(glfw::WindowHint::ContextVersion(3,2));
		glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
		
		if cfg!(macos) {
			#[cfg(target_os = "macos")]
				glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));
		}
		
		glfw.window_hint(glfw::WindowHint::OpenGlDebugContext(true));
		
		let initial_size = (1024, 768);
		let initial_title = "TaleCraft".to_string();
		
		let (mut window, events) = glfw
			.create_window(
				initial_size.0,
				initial_size.1,
				initial_title.as_str(),
				glfw::WindowMode::Windowed
			)
			.expect("Failed to create GLFW window.");
		
		use glfw::Context;
		window.make_current();
		
		window.set_key_polling(true);
		window.set_mouse_button_polling(true);
		window.set_cursor_pos_polling(true);
		window.set_cursor_mode(glfw::CursorMode::Normal);
		window.set_framebuffer_size_polling(true);
		window.set_refresh_polling(true);
		
		window.set_size_limits(
			320, 225,
			glfw::ffi::DONT_CARE as u32,
			glfw::ffi::DONT_CARE as u32
		);
		
		// Center the clients primary window in the middle of the primary monitor.
		glfw.with_primary_monitor_mut(|_, primary| {
			if let Some(monitor) = primary {
				if let Some(video_mode) = monitor.get_video_mode() {
					let size = window.get_size();
					window.set_pos(
						(video_mode.width as i32/2) - (size.0/2),
						(video_mode.height as i32/2) - (size.1/2)
					);
				}
			}
		});
		
		// Load OpenGL function pointers from the tcge-client-gl crate.
		let mut gl = gl::Gl::load_with(|symbol|
			window.get_proc_address(symbol) as *const _
		);
		
		let gl_info = unsafe {
			let version  = gl.GetString(gl::VERSION)  as *const i8;
			let vendor   = gl.GetString(gl::VENDOR)   as *const i8;
			let renderer = gl.GetString(gl::RENDERER) as *const i8;
			
			let version  = std::ffi::CStr::from_ptr(version ).to_str().expect("OpenGL version string" ).to_string();
			let vendor   = std::ffi::CStr::from_ptr(vendor  ).to_str().expect("OpenGL vendor string"  ).to_string();
			let renderer = std::ffi::CStr::from_ptr(renderer).to_str().expect("OpenGL renderer string").to_string();
			
			GlInfo {
				version,
				vendor,
				renderer
			}
		};
		
		info!("GL Version:  {}", gl_info.version);
		info!("GL Vendor:   {}", gl_info.vendor);
		info!("GL Renderer: {}", gl_info.renderer);
		
		// Enable debugging!
		gl.debug(on_gl_error, gl::DEBUG_SEVERITY_LOW);
		
		unsafe {
			gl.ClearColor(0.0, 0.0, 0.0, 0.0);
			gl.FrontFace(gl::CW);
			gl.CullFace(gl::BACK);
		}
		
		glfw.set_swap_interval(glfw::SwapInterval::Adaptive);
		
		window.swap_buffers();
		
		Self {
			title_fin: initial_title,
			title_dyn: String::with_capacity(32),
			glfw,
			window,
			events,
			gl,
			gl_info,
		}
	}
	
	pub fn completion(&self) -> bool {
		self.window.should_close()
	}
	
	pub fn update(&mut self) {
		self.glfw.poll_events();
	}
}



extern "system" fn on_gl_error(
	source: gl::types::GLenum,
	type_: gl::types::GLenum,
	id: gl::types::GLuint,
	severity: gl::types::GLenum,
	_length: gl::types::GLsizei,
	message: *const gl::types::GLchar,
	_user_val: *mut std::ffi::c_void,
) {
	if severity != gl::DEBUG_SEVERITY_NOTIFICATION {
		unsafe {
			let msg = std::ffi::CStr::from_ptr(message)
				.to_str().expect("Could not convert GL-Error to &str.");
			
			let source: &str = match source {
				gl::DEBUG_SOURCE_API             => "API",
				gl::DEBUG_SOURCE_APPLICATION     => "APPLICATION",
				gl::DEBUG_SOURCE_THIRD_PARTY     => "THIRD_PARTY",
				gl::DEBUG_SOURCE_WINDOW_SYSTEM   => "WINDOW_SYSTEM",
				gl::DEBUG_SOURCE_SHADER_COMPILER => "SHADER_COMPILER",
				gl::DEBUG_SOURCE_OTHER           => "OTHER",
				_ => "GL_DEBUG_SOURCE_UNKNOWN",
			};
			
			let type_: &str = match type_ {
				gl::DEBUG_TYPE_ERROR               => "TYPE_ERROR",
				gl::DEBUG_TYPE_DEPRECATED_BEHAVIOR => "DEPRECATED_BEHAVIOR",
				gl::DEBUG_TYPE_UNDEFINED_BEHAVIOR  => "UNDEFINED_BEHAVIOR",
				gl::DEBUG_TYPE_PORTABILITY         => "PORTABILITY",
				gl::DEBUG_TYPE_PERFORMANCE         => "PERFORMANCE",
				gl::DEBUG_TYPE_MARKER              => "MARKER",
				gl::DEBUG_TYPE_PUSH_GROUP          => "PUSH_GROUP",
				gl::DEBUG_TYPE_POP_GROUP           => "POP_GROUP",
				gl::DEBUG_TYPE_OTHER               => "OTHER",
				_ => "UNKNOWN",
			};
			
			match severity {
				gl::DEBUG_SEVERITY_HIGH => {
					eprintln!("[OpenGL/HIGH] {} threw {}, #{}: {}", source, type_, id, msg);
				},
				gl::DEBUG_SEVERITY_MEDIUM => {
					eprintln!("[OpenGL/MEDIUM] {} threw {}, #{}: {}", source, type_, id, msg);
				},
				gl::DEBUG_SEVERITY_LOW => {
					eprintln!("[OpenGL/LOW] {} threw {}, #{}: {}", source, type_, id, msg);
				},
				gl::DEBUG_SEVERITY_NOTIFICATION => {
					eprintln!("[OpenGL/INFO] {} threw {}, #{}: {}", source, type_, id, msg);
				},
				_ => (),
			}
		}
	}
}



impl backbone::Component for GlfwContext {
	fn get_type_name(&self) -> &'static str {
		"GlfwContext"
	}
	
	fn on_attachment(&mut self, _node_id: backbone::NodeId) {}
	
	fn on_detachment(&mut self, _node_id: backbone::NodeId) {}
	
	fn on_load(&mut self) {}
	
	fn on_unload(&mut self) {}
}



////////////////////////////////////////////////////////////////////////////////



pub struct ResizeEvent {
	pub width: i32,
	pub height: i32,
}

impl backbone::Event for ResizeEvent {
	fn get_type_name(&self) -> &'static str {
		"ResizeEvent"
	}
}

pub struct KeyEvent {
	pub key: glfw::Key,
	pub scancode: glfw::Scancode,
	pub action: glfw::Action,
	pub modifiers: glfw::Modifiers,
}

impl backbone::Event for KeyEvent {
	fn get_type_name(&self) -> &'static str {
		"KeyEvent"
	}
}

pub struct MouseEvent {
	pub button: glfw::MouseButton,
	pub action: glfw::Action,
	pub modifiers: glfw::Modifiers,
}

impl backbone::Event for MouseEvent {
	fn get_type_name(&self) -> &'static str {
		"KeyEvent"
	}
}

pub struct MouseMoveEvent {
	pub x: f64,
	pub y: f64,
	pub dx: f64,
	pub dy: f64,
}

impl backbone::Event for MouseMoveEvent {
	fn get_type_name(&self) -> &'static str {
		"MouseMoveEvent"
	}
}




////////////////////////////////////////////////////////////////////////////////

pub struct GlInfo {
	pub version: String,
	pub vendor: String,
	pub renderer: String,
}