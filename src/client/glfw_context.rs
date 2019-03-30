use super::super::router;
use super::cmd_opts;
use super::scene;

extern crate glfw;
use self::glfw::{Context, Key, Action};
use std::sync::mpsc::Receiver;
use std::cell::RefMut;
use std::ops::DerefMut;

pub struct GlfwContextComponent {
	pub glfw: glfw::Glfw,
	pub window: glfw::Window,
	pub events: Receiver<(f64, glfw::WindowEvent)>,
	cursor: Cursor,
}

impl GlfwContextComponent {
	pub fn new(opts: &cmd_opts::CmdOptions) -> Result<GlfwContextComponent, glfw::InitError> {
		let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS)?;
		
		glfw.window_hint(glfw::WindowHint::ContextVersion(3,2));
		glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
		
		#[cfg(target_os = "macos")]
			glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));
		glfw.window_hint(glfw::WindowHint::OpenGlDebugContext(true));
		glfw.window_hint(glfw::WindowHint::Samples(Some(opts.gl_multisamples)));
		
		// ------------------------------------------
		let window_title = format!("Talecraft Client: {}", env!("VERSION"));
		let (mut window, events) = glfw.create_window(
			opts.width,
			opts.height,
			&window_title,
			glfw::WindowMode::Windowed
		).expect("Failed to create GLFW window.");
		
		window.make_current();
		window.set_key_polling(true);
		window.set_cursor_pos_polling(true);
		window.set_cursor_mode(glfw::CursorMode::Normal);
		window.set_framebuffer_size_polling(true);
		window.set_size_limits(
			320, 225,
			glfw::ffi::DONT_CARE as u32,
			glfw::ffi::DONT_CARE as u32
		);
		
		// Center the clients primary window in the middle of the primary monitor.
		glfw.with_primary_monitor_mut(|_, primary| {
			if let Some(monitor) = primary {
				if let Some(vidmod) = monitor.get_video_mode() {
					debug!("Centering window on monitor: {}", monitor.get_name());
					let w_size = window.get_size();
					window.set_pos(
						(vidmod.width as i32/2) - (w_size.0/2),
						(vidmod.height as i32/2) - (w_size.1/2)
					);
				}
			}
		});
		
		// ------------------------------------------
		debug!("Loading OpenGL function-pointers...");
		gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);
		
		// ------------------------------------------
		// Only enable debugging if asked for...
		if opts.gl_debug {
			info!("OpenGL debugging is ENABLED.");
			unsafe {
				gl::Enable(gl::DEBUG_OUTPUT);
				gl::DebugMessageCallback(on_gl_error, 0 as *const std::ffi::c_void);
			}
		}
		
		// ------------------------------------------
		info!("Initialized window!");
		
		let cursor = Cursor {pos_x: 0.0, pos_y: 0.0, mov_x: 0.0, mov_y: 0.0};
		
		Ok(GlfwContextComponent {
			glfw,
			window,
			events,
			cursor,
		})
	}
	
	pub fn process_events(&mut self, router: &mut RefMut<router::Router>) {
		let router = router.deref_mut();
		
		for(_, event) in glfw::flush_messages(&mut self.events) {
			match event {
				glfw::WindowEvent::FramebufferSize(width, height) => {
					trace!("Resizing viewport to {}x{}", width, height);
					unsafe {gl::Viewport(0, 0, width, height)}
				},
				
				glfw::WindowEvent::Key(Key::M, _, Action::Press, _) => {
					if self.window.get_cursor_mode() == glfw::CursorMode::Disabled {
						self.window.set_cursor_mode(glfw::CursorMode::Normal);
						info!("Enabled mouse.");
					} else {
						self.window.set_cursor_mode(glfw::CursorMode::Disabled);
						info!("Disabled mouse.");
					}
					
					match router.nodes.get_mut_node_component_downcast::<scene::Scene>(0) {
						Ok(scene) => {
							scene.camera.active = self.window.get_cursor_mode() == glfw::CursorMode::Disabled
						},
						Err(_) => ()
					}
				},
				
				glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
					info!("User pressed ESC, shutting down...");
					self.window.set_should_close(true)
				},
				
				glfw::WindowEvent::CursorPos(x, y) => {
					self.cursor.update(x, y);
					
					match router.nodes.get_mut_node_component_downcast::<scene::Scene>(0) {
						Ok(scene) => {
							scene.camera.update_rotation(
								self.cursor.mov_x,
								self.cursor.mov_y
							);
						},
						Err(_) => ()
					}
				},
				_ => ()
			}
		}
	}
}

impl router::comp::Component for GlfwContextComponent {
	fn get_type_name(&self) -> &'static str {
		"GraphicsContext"
	}
	
	fn on_attachment(&mut self, _node_id: usize) {}
	fn on_detachment(&mut self, _node_id: usize) {}
	
	fn on_load(&mut self) {}
	fn on_unload(&mut self) {}
	
	fn on_event(&mut self, _event: &mut router::event::Wrapper) {
		//
	}
}

extern "system" fn on_gl_error(
	source: gl::types::GLenum,
	etype: gl::types::GLenum,
	id: gl::types::GLuint,
	severity: gl::types::GLenum,
	_length: gl::types::GLsizei,
	message: *const gl::types::GLchar,
	_userval: *mut std::ffi::c_void,
) {
	if severity != gl::DEBUG_SEVERITY_NOTIFICATION {
		unsafe {
			let msg = std::ffi::CStr::from_ptr(message)
				.to_str().expect("Could not convert GL-Error to &str.");
			error!("GL CALLBACK [{}, #{}, @{}, !{}]: {}", etype, id, source, severity, msg);
		}
	}
}

struct Cursor {
	pos_x: f32,
	pos_y: f32,
	mov_x: f32,
	mov_y: f32,
}

impl Cursor {
	fn update(&mut self, x: f64, y: f64) {
		self.mov_x = (x as f32) - self.pos_x;
		self.mov_y = (y as f32) - self.pos_y;
		self.pos_x = x as f32;
		self.pos_y = y as f32;
	}
}