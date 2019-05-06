//! Module for managing the game-window and associated events (mostly GLFW).

use super::super::router;
use super::cmd_opts;
use super::scene;

use super::glfw::{Context, Key, MouseButton, Action};
use std::sync::mpsc::Receiver;
use std::cell::RefMut;
use std::ops::DerefMut;

pub struct GlfwContextComponent {
	pub glfw: glfw::Glfw,
	pub window: glfw::Window,
	pub events: Receiver<(f64, glfw::WindowEvent)>,
	pub last_esc: u128,
	cursor: Cursor,
}

impl GlfwContextComponent {
	pub fn new(opts: &cmd_opts::CmdOptions) -> Result<GlfwContextComponent, glfw::InitError> {
		let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS)?;
		
		glfw.window_hint(glfw::WindowHint::ContextVersion(3,2));
		glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
		
		if cfg!(macos) {
			#[cfg(target_os = "macos")]
			glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));
		}
		
		if opts.gl_multisamples != 0 {
			glfw.window_hint(glfw::WindowHint::Samples(Some(opts.gl_multisamples)));
		}
		
		if opts.gl_debug {
			glfw.window_hint(glfw::WindowHint::OpenGlDebugContext(true));
		}
		
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
		window.set_mouse_button_polling(true);
		window.set_cursor_pos_polling(true);
		window.set_cursor_mode(glfw::CursorMode::Normal);
		window.set_framebuffer_size_polling(true);
		window.set_size_limits(
			320, 225,
			glfw::ffi::DONT_CARE as u32,
			glfw::ffi::DONT_CARE as u32
		);
		
		let initial_window_size = window.get_size();
		let initial_framebuf_size = window.get_framebuffer_size();
		let initial_content_scale = window.get_content_scale();
		
		debug!("Initial Window Size:   {} {}", initial_window_size.0, initial_window_size.1);
		debug!("Initial Frame Size:    {} {}", initial_framebuf_size.0, initial_framebuf_size.1);
		debug!("Initial Content Scale: {} {}", initial_content_scale.0, initial_content_scale.1);
		
		// Center the clients primary window in the middle of the primary monitor.
		glfw.with_primary_monitor_mut(|_, primary| {
			if let Some(monitor) = primary {
				if let Some(vidmod) = monitor.get_video_mode() {
					debug!("Centering window on monitor '{}' ...", monitor.get_name().unwrap_or(String::from("[UNKNOWN]")));
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
		
		let cursor = Cursor::new();
		
		Ok(GlfwContextComponent {
			glfw,
			window,
			events,
			cursor,
			last_esc: super::super::util::current_time_nanos()
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
				
				glfw::WindowEvent::Key(Key::M, _, Action::Release, _) => {
					let new_state = GlfwContextComponent::toggle_cursor_mode(
						&mut self.window,
						None // toggle
					);
					
					match router.nodes.get_mut_node_component_downcast::<scene::Scene>(0) {
						Ok(scene) => {
							scene.camera.active = new_state == glfw::CursorMode::Disabled
						},
						Err(_) => ()
					}
				},

				glfw::WindowEvent::Key(Key::C, _, Action::Release, _) => {
					match router.nodes.get_mut_node_component_downcast::<scene::Scene>(0) {
						Ok(scene) => {
							scene.camera.crane = !scene.camera.crane;
						},
						Err(_) => ()
					}
				},
				
				glfw::WindowEvent::Key(Key::F5, _, Action::Release, _) => {
					info!("User pressed R, reloading settings...");
					if let Ok(settings) = router.nodes.get_mut_node_component_downcast::<super::settings::Settings>(0) {
						if let Ok(_) = settings.load() {
							router.fire_event_at_lens("client", &mut super::settings::SettingsReloadEvent::new(settings));
						}
					}
				},
				
				glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
					let current = super::super::util::current_time_nanos();
					
					if (current - self.last_esc) < 500000000 {
						info!("User pressed ESC twice, shutting down...");
						self.window.set_should_close(true)
					} else {
						info!("User pressed ESC once...");
						self.last_esc = current;
					}
				},
				
				glfw::WindowEvent::Key(Key::Num1, _, Action::Press, _) => {
					if let Ok(scene) = router.nodes.get_mut_node_component_downcast::<scene::Scene>(0) {
						scene.camera.block = Some(scene.blockdef.get_block_by_name_unchecked("bedrock").get_default_state());
					}
				},
				
				glfw::WindowEvent::Key(Key::Num2, _, Action::Press, _) => {
					if let Ok(scene) = router.nodes.get_mut_node_component_downcast::<scene::Scene>(0) {
						scene.camera.block = Some(scene.blockdef.get_block_by_name_unchecked("bedrock2").get_default_state());
					}
				},
				
				glfw::WindowEvent::Key(Key::Num3, _, Action::Press, _) => {
					if let Ok(scene) = router.nodes.get_mut_node_component_downcast::<scene::Scene>(0) {
						scene.camera.block = Some(scene.blockdef.get_block_by_name_unchecked("bedrock3").get_default_state());
					}
				},
				
				glfw::WindowEvent::MouseButton(button, Action::Press, _) => {
					if self.window.get_cursor_mode() != glfw::CursorMode::Disabled {
						continue;
					}
					
					match router.nodes.get_mut_node_component_downcast::<scene::Scene>(0) {
						Ok(scene) => {
							let src = scene.camera.get_position(1.0);
							let dir = scene.camera.get_look_dir(1.0);
							let len = 16.0;
							
							use super::blocks;
							let mut rc = blocks::BlockRaycast::new_from_src_dir_len(src, dir, len);
							
							let air = scene.blockdef
								.get_block_by_name_unchecked("air")
								.get_default_state();
							
							let bedrock = scene.blockdef
								.get_block_by_name_unchecked("bedrock")
								.get_default_state();
							
							let used_block = scene.camera.block.unwrap_or(bedrock);
							
							match scene.chunks.raycast(&mut rc) {
								Some((last_pos, curr_pos, _block)) => {
									match button {
										MouseButton::Button1 => {
											scene.chunks.set_block(&curr_pos, air);
										},
										MouseButton::Button2 => {
											scene.chunks.set_block(&last_pos, used_block);
										},
										_ => {}
									}
								},
								None => ()
							}
						},
						Err(_) => ()
					}
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
	
	pub fn toggle_cursor_mode(window: &mut glfw::Window, state: Option<glfw::CursorMode>) -> glfw::CursorMode {
		// Direct state change
		if let Some(state) = state {
			GlfwContextComponent::set_cursor_mode(window, state);
			return state;
		}
		
		// Toggle state change
		let state = match window.get_cursor_mode() {
			glfw::CursorMode::Normal => glfw::CursorMode::Disabled,
			glfw::CursorMode::Hidden => glfw::CursorMode::Disabled,
			glfw::CursorMode::Disabled => glfw::CursorMode::Normal,
		};
		
		GlfwContextComponent::set_cursor_mode(window, state);
		return state;
	}
	
	fn set_cursor_mode(window: &mut glfw::Window, state: glfw::CursorMode) {
		window.set_cursor_mode(state);
		match state {
			glfw::CursorMode::Disabled => {info!("Disabled mouse.")},
			glfw::CursorMode::Normal => {info!("Enabled mouse.");}
			glfw::CursorMode::Hidden => {info!("Enabled mouse.");}
		}
	}
	
	pub fn swap_and_poll(&mut self) {
		self.window.swap_buffers();
		self.glfw.poll_events();
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
	fn new() -> Cursor {
		Cursor {
			pos_x: 0.0,
			pos_y: 0.0,
			mov_x: 0.0,
			mov_y: 0.0,
		}
	}
	
	fn update(&mut self, x: f64, y: f64) {
		self.mov_x = (x as f32) - self.pos_x;
		self.mov_y = (y as f32) - self.pos_y;
		self.pos_x = x as f32;
		self.pos_y = y as f32;
	}
}