#[macro_use] extern crate log;
#[macro_use] extern crate smallvec;
extern crate simplelog;
extern crate rustc_hash;
extern crate cgmath;

extern crate glfw;
extern crate tcge_common as common;
extern crate tcge_backbone as backbone;
extern crate tcge_blocks as blocks;
extern crate tcge_opengl as gl;

use common::gameloop;
use backbone::WrapperComponent;

mod glfw_context;
use glfw_context::GlfwContext;
use glfw_context::{ResizeEvent, MouseMoveEvent, MouseEvent, KeyEvent};
use common::resources::ResourceProvider;

mod render;
mod playground;

const DEFAULT_TICKS_PER_SECOND: i32 = 60;

fn main() {
	init_logger();
	info!("Hello, world!");
	
	let mut backbone = backbone::Backbone::new();
	let root_id = backbone.root_get_id();
	
	let root_node_handler = RootNodeHandler {};
	let root_node_handler = Box::new(root_node_handler );
	backbone.set_root_node_handler(root_node_handler);
	
	// Attach GlfwContext and get a mutable reference to it...
	let glfw_context = backbone.node_component_attach(root_id, GlfwContext::new());
	
	let resources = common::resources::new();
	let resources = WrapperComponent::new("Resources", resources);
	let resources = backbone.node_component_attach(root_id, resources);
	
	// Register all default (core) resources embedded in the binary
	let mut includes: common::resources::Includes = vec![];
	includes.extend(&render::text::TEXT_RENDERER_FILES);
	includes.extend(&render::materials::SOLID_COLOR_MATERIAL_FILES);
	includes.extend(&playground::sky::SKY_MATERIAL_FILES);
	includes.extend(&playground::grid::GRID_MATERIAL_FILES);
	includes.extend(&playground::test_blocks::BLOCKS_MATERIAL_FILES);
	
	// Register the embedded files.
	let includes = common::resources::IncludeProvider::new(includes);
	resources.register_provider_by_type(includes);
	
	for iterator in resources.res_list() {
		for path in iterator {
			info!("Found Resource: {}", path);
		}
	}
	
	// Setup the text renderer
	let text_renderer = render::text::new(&glfw_context.gl, resources, "hack")
		.ok().expect("TextRenderer initialization failed.");
	let text_renderer = WrapperComponent::new("TextRenderer", text_renderer);
	backbone.node_component_attach(root_id, text_renderer);
	
	
	// Setup the playground.
	// TODO: Eventually move this into the initializer.
	playground::setup(
		&mut backbone,
		glfw_context,
		resources
	);
	
	backbone.location_set("/playground").unwrap();
	
	// Show the window and wait until things calm down.
	glfw_context.window.show();
	glfw_context.gl.flush();
	backbone.update_until_idle();
	
	// Create the gameloop, attach it to the backbone, then run it.
	let gameloop = gameloop::new(DEFAULT_TICKS_PER_SECOND);
	let gameloop = WrapperComponent::new("Gameloop", gameloop);
	let gameloop = &mut **backbone.node_component_attach(root_id, gameloop);
	
	info!("Starting gameloop with {} ticks per second.", gameloop.get_ticks_per_second());
	
	main_loop(&mut backbone, glfw_context, gameloop);
	
	// The End.
	info!("Goodbye, world!");
}

fn init_logger() {
	use simplelog::*;
	let current_exe = std::env::current_exe().expect("Failed to get path of the 'client' executable.");
	let current_dir = current_exe.parent().expect("Failed to get path of the 'client' executables parent directory.");
	let log_file = current_dir.join("client.log");
	let mut log_config = Config::default();
	log_config.time_format = Some("[%Y-%m-%d %H:%M:%S]");
	
	println!("[HINT] Log file location: {}", log_file.to_str().unwrap_or("ERROR"));
	let log_file = std::fs::File::create(log_file).expect("Failed to set up FileLogger for client.");
	
	CombinedLogger::init(
		vec![
			TermLogger::new(LevelFilter::Trace, log_config, TerminalMode::Mixed).unwrap(),
			WriteLogger::new(LevelFilter::Info, log_config, log_file),
		]
	).expect("Failed to initialize simplelog::CombinedLogger");
}

fn main_loop(
	backbone: &mut backbone::Backbone,
	glfw_context: &mut GlfwContext,
	gameloop: &mut gameloop::State,
) {
	let command_line = common::commandline::CommandLine::new();
	let mut last_escape_press = common::current_time_nanos();
	let mut cursor = (0.0, 0.0);
	
	while backbone.update() {
		
		if glfw_context.completion() {
			backbone.stop();
			continue
		}
		
		while let Some(command) = command_line.recv() {
			backbone.fire_event(&mut CommandEvent {
				command
			});
		}
		
		glfw_context.update();
		
		let loop_state = gameloop.update(
			|| glfw_context.glfw.get_time()
		);
		
		for (_time, event) in glfw::flush_messages(&glfw_context.events) {
			match event {
				glfw::WindowEvent::Key(glfw::Key::Escape, _, glfw::Action::Press, _) => {
					let current = common::current_time_nanos();
					
					if (current - last_escape_press) < 500000000 {
						info!("User pressed ESC twice, shutting down...");
						glfw_context.window.set_should_close(true)
					} else {
						info!("User pressed ESC once...");
						last_escape_press = current;
					}
				},
				
				glfw::WindowEvent::Key(key, scancode, action, modifiers) => {
					backbone.fire_event(&mut KeyEvent {
						key, scancode, action, modifiers
					});
				},
				
				glfw::WindowEvent::MouseButton(button, action, modifiers) => {
					backbone.fire_event(&mut MouseEvent {
						button, action, modifiers
					});
				},
				
				glfw::WindowEvent::CursorPos(x, y) => {
					// calc delta
					let dx = x - cursor.0;
					let dy = y - cursor.1;
					
					// fire event
					backbone.fire_event(&mut MouseMoveEvent {
						x, y, dx, dy
					});
					
					// update
					cursor.0 = x;
					cursor.1 = y;
				},
				
				glfw::WindowEvent::FramebufferSize(width, height) => {
					unsafe {
						glfw_context.gl.Viewport(0, 0, width, height);
					}
					
					backbone.fire_event(&mut ResizeEvent {
						width, height
					});
				},
				
				glfw::WindowEvent::Refresh => {
					// TODO: Figure out why this doesn't work?
					/*
					backbone.fire_event(&mut RenderEvent {
						time, interpolation: 0.0,
					});
					*/
				}
				
				_ => {}
			}
		}
		
		match loop_state {
			gameloop::Tick(tps, time, delta) => {
				backbone.fire_event(&mut TickEvent {
					tps, time, delta
				});
			},
			
			gameloop::Frame(time, interpolation) => {
				let window_size = glfw_context.window.get_framebuffer_size();
				backbone.fire_event(&mut RenderEvent {
					gl: glfw_context.gl.clone(),
					time,
					width:  window_size.0,
					height: window_size.1,
					interpolation: interpolation as f32,
					delta: 1.0 / gameloop.get_ticks_per_second() as f32,
				});
			},
			
			gameloop::Timer(fps, tps) => {
				use std::fmt::Write;
				glfw_context.title_dyn.clear();
				write!(
					glfw_context.title_dyn,
					"{} - {:.0} FPS, {:.0} TPS - {}",
					glfw_context.title_fin,
					fps, tps,
					backbone.location_get_str()
				).unwrap();
				glfw_context.window.set_title(glfw_context.title_dyn.as_str());
			},
			
			gameloop::Stop => {
				glfw_context.window.set_should_close(true)
			}
			
			_ => ()
		}
	}
	
}



pub struct TickEvent {
	pub tps: i32,
	pub time: f64,
	pub delta: f32,
}

impl backbone::Event for TickEvent {
	fn get_type_name(&self) -> &'static str {
		"TickEvent"
	}
}

pub struct RenderEvent {
	pub gl: gl::Gl,
	pub time: f64,
	pub interpolation: f32,
	pub delta: f32,
	pub width:  i32,
	pub height: i32,
}

impl backbone::Event for RenderEvent {
	fn get_type_name(&self) -> &'static str {
		"RenderEvent"
	}
}

pub struct CommandEvent {
	pub command: String
}

impl backbone::Event for CommandEvent {
	fn get_type_name(&self) -> &'static str {
		"CommandEvent"
	}
}

struct RootNodeHandler {}

impl backbone::Handler for RootNodeHandler {
	fn on_event<'a>(&mut self, event: &mut backbone::Wrapper, context: &mut backbone::Context) {
		let phase = event.get_phase().clone();
		
		let glfw_context = context
			.component_get_mut::<GlfwContext>()
			.ok().unwrap();
		
		if let Some(cmd) = event.downcast_mut::<CommandEvent>() {
			// 'steal' the command from the struct to avoid the borrow checker
			let command = std::mem::replace(&mut cmd.command, String::new());
			
			if command == "stop" {
				event.stop();
				context.component_get_mut::<WrapperComponent<gameloop::State>>()
					.map(|gameloop| {
						gameloop.stop();
					}).ok();
				event.stop();
				return;
			}
			
			if command.starts_with("echo ") {
				if let Some(mid) = command.find(' ') {
					let (_, echo) = command.split_at(mid);
					let echo = echo.trim().to_string();
					info!("Echo: {}", echo);
				}
				event.stop();
				return;
			}
			
			if command.starts_with("loc ") {
				if let Some(mid) = command.find(' ') {
					let (_, path) = command.split_at(mid);
					let path = path.trim().to_string();
					info!("Attempting to move to path: {}", path);
					event.new_state(backbone::State::Move(path, 0));
				}
				event.stop();
				return;
			}
			
			if command.starts_with("set-tps ") {
				let mut tps = DEFAULT_TICKS_PER_SECOND;
				if let Some(mid) = command.find(' ') {
					let (_, num) = command.split_at(mid);
					let num = num.trim();
					match num.parse::<i32>() {
						Ok(num) => tps = num,
						Err(err) => {
							error!("Could not parse number '{}': {}", num, err);
							event.stop();
							return
						},
					}
				} else {
					error!("No tick-rate given, using default.");
				}
				
				info!("Changing tick-rate to {} tps.", tps);
				context.component_get_mut::<WrapperComponent<gameloop::State>>()
					.map(|gameloop| {
						gameloop.set_ticks_per_second(tps);
					}).ok();
				event.stop();
				return;
			}
			
			// give the command back to the struct
			cmd.command = command;
			return
		}
		
		if let Some(_) = event.downcast::<MouseMoveEvent>() {
			if glfw_context.window.get_cursor_mode() != glfw::CursorMode::Disabled {
				event.stop();
			}
			return
		}
		
		if let Some(_) = event.downcast::<MouseEvent>() {
			return
		}
		
		if let Some(key_event) = event.downcast::<KeyEvent>() {
			
			match key_event {
				KeyEvent{key: glfw::Key::M, scancode: _, action: glfw::Action::Press, modifiers: _} => {
					let new_state = toggle_cursor_mode(
						&mut glfw_context.window,
						None // toggle
					);
					glfw_context.window.set_cursor_mode(new_state);
					event.stop();
				},
				_ => (),
			}
			
			return
		}
		
		if let Some(_) = event.downcast::<TickEvent>() {
			return
		}
		
		if let Some(render_event) = event.downcast::<RenderEvent>() {
			match phase {
				backbone::Phase::Creation => {},
				
				backbone::Phase::Propagation => {
					unsafe {
						render_event.gl.ClearColor(0.0, 0.0, 0.0, 1.0);
						render_event.gl.Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT | gl::STENCIL_BUFFER_BIT);
					}
				},
				
				backbone::Phase::Action => {},
				
				backbone::Phase::Bubbling => {
					use glfw::Context;
					glfw_context.window.swap_buffers();
				},
			}
			
			return
		}
		
		debug!("RootNodeHandler received {} in {}-phase.", event.event.get_type_name(), event.get_phase());
	}
}

pub fn toggle_cursor_mode(window: &mut glfw::Window, state: Option<glfw::CursorMode>) -> glfw::CursorMode {
	// Direct state change
	if let Some(state) = state {
		return state;
	}
	
	// Toggle state change
	let state = match window.get_cursor_mode() {
		glfw::CursorMode::Normal => glfw::CursorMode::Disabled,
		glfw::CursorMode::Hidden => glfw::CursorMode::Disabled,
		glfw::CursorMode::Disabled => glfw::CursorMode::Normal,
	};
	
	return state;
}