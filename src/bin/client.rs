use std::rc::Rc;
use std::cell::RefCell;

#[macro_use]
extern crate log;
extern crate simplelog;

extern crate failure;
#[allow(unused_imports)]
use failure::Fail;

extern crate time;
extern crate glfw;
use glfw::{Context, Key, Action};
extern crate image;
extern crate gl;

extern crate tcge;
use tcge::resources;
use tcge::client::cmd_opts;
use tcge::client::render;
use tcge::client::geometry;
use tcge::client::freecam;
use tcge::gameloop;
use std::sync::mpsc::Receiver;

fn main() {
	let options = match cmd_opts::parse() {
		Err(e) => {
			print_error(&e);
			panic!("Failed to parse command-line arguments! Exiting...");
		}
		Ok(o) => o
	};
	
	use simplelog::*;
	use std::fs::File;
	let current_exe = std::env::current_exe().unwrap();
	let current_dir = current_exe.parent().unwrap();
	let log_file = current_dir.join("client.log");
	let mut log_config = Config::default();
	log_config.time_format = Some("[%Y-%m-%d %H:%M:%S]");
	
	CombinedLogger::init(
		vec![
			TermLogger::new(LevelFilter::Trace, log_config).unwrap(),
			WriteLogger::new(LevelFilter::Info, log_config, File::create(log_file).unwrap()),
		]
	).unwrap();
	
	if let Err(e) = run(options) {
		print_error(&e);
		panic!("A fatal error occurred and the engine had to stop...");
	}
	
	info!("Goodbye!\n");
}

fn print_error(e: &failure::Error) {
	use std::fmt::Write;
	let mut result = String::new();
	
	for (i, cause) in e.iter_chain().collect::<Vec<_>>().into_iter().enumerate() {
		if i > 0 {
			let _ = write!(&mut result, "   Caused by: ");
		}
		let _ = write!(&mut result, "{}", cause);
		if let Some(backtrace) = cause.backtrace() {
			let backtrace_str = format!("{}", backtrace);
			if !backtrace_str.is_empty() {
				let _ = writeln!(&mut result, " This happened at {}", backtrace);
			} else {
				let _ = writeln!(&mut result);
			}
		} else {
			let _ = writeln!(&mut result);
		}
	}
	
	error!("{}\n", result);
}

fn new_window(
	glfw: &mut glfw::Glfw,
	opts: &cmd_opts::CmdOptions
) -> (glfw::Window, Receiver<(f64, glfw::WindowEvent)>) {
	
	glfw.window_hint(glfw::WindowHint::ContextVersion(3,2));
	glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
	
	#[cfg(target_os = "macos")]
		glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));
	glfw.window_hint(glfw::WindowHint::OpenGlDebugContext(true));
	glfw.window_hint(glfw::WindowHint::Samples(Some(opts.gl_multisamples)));
	
	// ------------------------------------------
	let (mut window, events) = glfw.create_window(
		1024, 768, "Talecraft",
		glfw::WindowMode::Windowed
	).expect("Failed to create GLFW window.");
	
	window.make_current();
	window.set_key_polling(true);
	window.set_cursor_pos_polling(true);
	window.set_cursor_mode(glfw::CursorMode::Disabled);
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
				let w_size = window.get_size();
				window.set_pos(
					(vidmod.width as i32/2) - (w_size.0/2),
					(vidmod.height as i32/2) - (w_size.1/2)
				);
			}
		}
	});
	
	// ------------------------------------------
	gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);
	
	// ------------------------------------------
	// Only enable debugging if asked for...
	if opts.gl_debug {
		unsafe {
			gl::Enable(gl::DEBUG_OUTPUT);
			gl::DebugMessageCallback(on_gl_error, 0 as *const std::ffi::c_void);
		}
	}
	
	// ------------------------------------------
	info!("Initialized window!");
	return (window, events);
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

fn run(opts: cmd_opts::CmdOptions) -> Result<(), failure::Error> {
	// ------------------------------------------
	let res = resources::Resources::from_exe_path()?;
	
	// ------------------------------------------
	let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS)?;
	let (mut window, events) = new_window(&mut glfw, &opts);
	
	/*
	unsafe {
		let depth_bits = glfw::ffi::glfwGetWindowAttrib(window.window_ptr(), glfw::ffi::DEPTH_BITS);
		let depth_bits = gl::GetFramebufferAttachmentParameteriv(gl::FRAMEBUFFER, gl::GL_DEPTH_ATTACHMENT);
		debug!("Available depth bits: {}", depth_bits);
	}
	*/
	
	// ------------------------------------------
	let shader_grid = render::materials::ShaderGrid::new(&res)?;
	let shader_random = render::materials::ShaderRandom::new(&res)?;
	
	let ascii_renderer = render::ascii_text::AsciiTextRenderer::load(&res)?;
	
	// ------------------------------------------
	let mut render_state_gui = GuiRenderState {
		width: 0, height: 0,
		ascii_renderer,
		frame_time: 0.0,
		last_fps: 0.0,
		last_tps: 0.0,
	};
	
	// ------------------------------------------
	let mut render_state = RenderState {
		frame_id: 0,
		shader_grid,
		shader_random,
	};
	
	let mut cursor = Cursor {pos_x: 0.0, pos_y: 0.0, mov_x: 0.0, mov_y: 0.0};
	
	info!("Initializing scene...");
	
	let scene = Rc::new(RefCell::new(Option::Some(Scene {
		camera: freecam::Camera::new(),
		meshes: vec![
			geometry::geometry_test(),
			geometry::geometry_cube(1.0),
			// geometry::geometry_cube(-512.0),
		],
		mesh_planequad: geometry::geometry_planequad(1024.0),
	})));
	
	// ------------------------------------------
	info!("Initializing gameloop...");
	
	let mut gls = gameloop::GameloopState::new(30, true);
	
	info!("Starting gameloop...");
	while !window.should_close() {
		process_events(
			&mut window,
			&events,
			&mut cursor,
			&mut *scene.borrow_mut()
		);
		
		let window_size = window.get_framebuffer_size();
		let mut reset_render_state = false;
		let frame_time  = gls.get_frame_time();
		let last_fps = gls.get_frames_per_second();
		let last_tps = gls.get_ticks_per_second();
		
		gls.next(|| {glfw.get_time()},
			
			|_now:f64| {
				scene.borrow_mut().as_mut().map(|mut_scene| {
					mut_scene.camera.update_movement(&window);
				});
				
				reset_render_state = true;
			},
			
			|now: f64, interpolation: f32| {
				unsafe {
					gl::Clear(gl::COLOR_BUFFER_BIT
							| gl::DEPTH_BUFFER_BIT
							| gl::STENCIL_BUFFER_BIT
					);
				}
				
				(&mut render_state).begin();
				scene.borrow().as_ref().map(|scene| {
					render(
						&render_state,
						&scene,
						&scene.camera,
						window_size,
						now,
						interpolation
					);
				});
				(&mut render_state).end();
				
				let (w, h) = window.get_framebuffer_size();
				render_state_gui.width = w;
				render_state_gui.height = h;
				render_state_gui.frame_time = frame_time;
				render_state_gui.last_fps = last_fps;
				render_state_gui.last_tps = last_tps;
				render_gui(&mut render_state_gui);
			}
		);
		
		if reset_render_state {
			(&mut render_state).reset();
		}
		
		window.swap_buffers();
		glfw.poll_events();
	}
	
	Ok(())
}

fn process_events(
	window: &mut glfw::Window,
	events: &std::sync::mpsc::Receiver<(f64, glfw::WindowEvent)>,
	cursor: &mut Cursor,
	opt_scene: &mut Option<Scene>
) {
	for(_, event) in glfw::flush_messages(events) {
		match event {
			glfw::WindowEvent::FramebufferSize(width, height) => {
				unsafe {gl::Viewport(0, 0, width, height)}
			},
			
			glfw::WindowEvent::Key(Key::M, _, Action::Press, _) => {
				if window.get_cursor_mode() == glfw::CursorMode::Disabled {
					window.set_cursor_mode(glfw::CursorMode::Normal);
					info!("Enabled mouse.");
				} else {
					window.set_cursor_mode(glfw::CursorMode::Disabled);
					info!("Disabled mouse.");
				}
				
				opt_scene.as_mut()
					.map(|mut_scene| &mut mut_scene.camera)
					.map( |mut_camera| {
						mut_camera.active = window.get_cursor_mode() == glfw::CursorMode::Disabled;
					});
			},
			
			glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
				window.set_should_close(true)
			},
			
			glfw::WindowEvent::CursorPos(x, y) => {
				cursor.update(x, y);
				opt_scene.as_mut()
					.map(|mut_scene| &mut mut_scene.camera)
					.map( |mut_camera| {
						mut_camera.update_rotation(
							cursor.mov_x,
							cursor.mov_y
						);
					});
			},
			_ => ()
		}
	}
}

struct Scene {
	camera: freecam::Camera,
	meshes: Vec<geometry::SimpleVao>,
	mesh_planequad: geometry::SimpleVao,
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

struct RenderState {
	frame_id: i64,
	shader_grid: render::materials::ShaderGrid,
	shader_random: render::materials::ShaderRandom,
}

impl RenderState {
	fn begin(&mut self) {
		self.frame_id = self.frame_id + 1;
	}
	fn end(&mut self) {}
	
	fn reset(&mut self) {
		self.frame_id = 0;
	}
}

fn render(render_state: &RenderState, scene: &Scene, camera: &freecam::Camera, size: (i32, i32), now: f64, _interpolation:f32) {
	render::utility::gl_push_debug("Draw Scene");
	
	unsafe {
		gl::Enable(gl::DEPTH_TEST);
		gl::CullFace(gl::FRONT);
		gl::Enable(gl::CULL_FACE);
	}
	
	let camera_transform = camera.transform(size, _interpolation, true);
	
	render::utility::gl_push_debug("Draw Grid");
	{
		unsafe {
			gl::Enable(gl::BLEND);
			gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
			gl::Disable(gl::DEPTH_TEST);
		}
		let shader_grid = &render_state.shader_grid;
		shader_grid.shader_program.set_used();
		shader_grid.shader_program.uniform_matrix4(shader_grid.uniform_matrix, camera_transform);
		scene.mesh_planequad.draw(gl::TRIANGLES);
		unsafe {
			gl::Enable(gl::DEPTH_TEST);
			gl::Disable(gl::BLEND);
		}
	}
	render::utility::gl_pop_debug();
	
	let shader_random = &render_state.shader_random;
	shader_random.shader_program.set_used();
	shader_random.shader_program.uniform_matrix4(shader_random.uniform_matrix, camera_transform);
	shader_random.shader_program.uniform_scalar(shader_random.uniform_time, now as f32);
	
	for mesh in scene.meshes.iter() {
		mesh.draw(gl::TRIANGLES);
	}
	
	render::utility::gl_pop_debug();
}

struct GuiRenderState {
	width: i32, height: i32,
	ascii_renderer: render::ascii_text::AsciiTextRenderer,
	frame_time: f64,
	last_fps: f64,
	last_tps: f64,
}

fn render_gui(render_state_gui: &mut GuiRenderState) {
	render::utility::gl_push_debug("Draw GUI");
	
	unsafe {
		gl::Flush();
		gl::Enable(gl::BLEND);
		gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
		gl::Disable(gl::DEPTH_TEST);
	}
	
	let projection = cgmath::ortho(0.0,
		render_state_gui.width as f32,
		render_state_gui.height as f32,
		0.0,
		-1.0,1.0
	);
	
	let frame_time = (render_state_gui.frame_time * 1000.0).ceil();
	let last_fps = render_state_gui.last_fps.floor();
	let last_tps = render_state_gui.last_tps.round(); // its impossible to get the exact TPS
	
	render_state_gui.ascii_renderer.transform = projection;
	render_state_gui.ascii_renderer.draw_text(
		format!("TCGE {}: {}ms ({} FPS, {} TPS)", env!("VERSION"), frame_time, last_fps, last_tps),
		16.0, 0.0+1.0, 16.0
	);
	
	render::utility::gl_pop_debug();
}