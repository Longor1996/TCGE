use std::rc::Rc;
use std::cell::{RefCell};
use core::borrow::Borrow;

#[macro_use]
extern crate log;
extern crate simplelog;

extern crate failure;
#[allow(unused_imports)]
use failure::Fail;

extern crate time;
extern crate glfw;
use glfw::{Context};
extern crate image;
extern crate gl;

extern crate tcge;
extern crate core;

use tcge::resources;
use tcge::router;
use tcge::gameloop;
use tcge::client::cmd_opts;
use tcge::client::glfw_context;
use tcge::client::scene;
use tcge::client::render;

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
	let current_exe = std::env::current_exe().expect("Failed to get path of the 'client' executable.");
	let current_dir = current_exe.parent().expect("Failed to get path of the 'client' executables parent directory.");
	let log_file = current_dir.join("client.log");
	let mut log_config = Config::default();
	log_config.time_format = Some("[%Y-%m-%d %H:%M:%S]");
	
	println!("[HINT] Log file location: {}", log_file.to_str().unwrap_or("ERROR"));
	CombinedLogger::init(
		vec![
			TermLogger::new(LevelFilter::Trace, log_config).expect("Failed to set up TermLogger for client."),
			WriteLogger::new(LevelFilter::Info, log_config, File::create(log_file).expect("Failed to set up FileLogger for client.")),
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

struct ClientLens {
	// Nothing here yet.
}

impl router::lens::Handler for ClientLens {
	fn on_event<'a>(
		&mut self,
		event: &mut router::event::Wrapper,
		context: &mut router::context::Context
	) -> router::lens::State {
		
		event.downcast::<TickEvent>().map(|_tick| {
			let s = context.get_mut_component_downcast::<scene::Scene>();
			let g = context.get_mut_component_downcast::<glfw_context::GlfwContextComponent>();
			
			match s {
				Ok(scene) => {
					match g {
						Ok(gfx_root) => {
							scene.camera.update_movement(gfx_root.window.borrow());
						},
						Err(_) => ()
					}
				}
				Err(_) => ()
			}
			
			match context.get_mut_component_downcast::<scene::SceneRenderState>() {
				Ok(scene_render_state) => {
					scene_render_state.reset();
				},
				Err(_) => ()
			};
		});
		
		event.downcast::<DrawEvent>().map(|draw| {
			let s = context.get_mut_component_downcast::<scene::Scene>();
			let sr = context.get_mut_component_downcast::<scene::SceneRenderState>();
			
			if s.is_err() {
				panic!("This ain't supposed to happen!");
			}
			
			match s {
				Ok(scene) => {
					match sr {
						Ok(scene_render_state) => {
							scene_render_state.begin();
							scene::render(
								scene_render_state,
								scene,
								draw.window_size,
								draw.now,
								draw.interpolation
							);
							scene_render_state.end();
						},
						Err(_) => ()
					}
				}
				Err(_) => ()
			}
		});
		
		router::lens::State::Idle
	}
}

fn run(opts: cmd_opts::CmdOptions) -> Result<(), failure::Error> {
	// ------------------------------------------
	let mut router = router::Router::new();
	let res = resources::Resources::from_exe_path()?;
	
	// ------------------------------------------
	let gfxroot = glfw_context::GlfwContextComponent::new(&opts)?;
	
	// Give the router ownership of the Graphics-Context... then sneakily grab it back!
	// This is the **only** place in the code where it's okay to do this.
	router.nodes.set_node_component(0, Box::new(gfxroot))?;
	let gfxroot = router.nodes
		.get_mut_node_component_downcast::<glfw_context::GlfwContextComponent>(0)?;
	
	// ------------------------------------------
	
	let ascii_renderer = render::text::AsciiTextRenderer::load(&res)?;
	let mut render_state_gui = GuiRenderState {
		width: 0, height: 0,
		ascii_renderer,
		debug_text: vec![],
	};
	
	// ------------------------------------------
	
	info!("Initializing scene...");
	
	let mut scene = scene::Scene::new();
	scene.camera.active = gfxroot.window.get_cursor_mode() == glfw::CursorMode::Disabled;
	router.nodes.set_node_component(0, Box::new(scene))?;
	
	let scene_render_state = scene::SceneRenderState::new(&res)?;
	router.nodes.set_node_component(0, Box::new(scene_render_state))?;
	
	// ------------------------------------------
	
	// Create the client lens...
	router.new_lens("client", &|_| {
		Some(Box::new(ClientLens {
			// nothing here yet
		}))
	});
	
	// Then put the router into a RefCell and (hopefully) never touch it again!
	let router = Rc::new(RefCell::new(router));
	
	// ------------------------------------------
	info!("Initializing and starting gameloop...");
	let mut gls = gameloop::GameloopState::new(30, true);
	
	while !router.borrow_mut().update() && !gfxroot.window.should_close() {
		gfxroot.process_events(&mut router.borrow_mut());
		
		let window_size = gfxroot.window.get_framebuffer_size();
		let frame_time  = gls.get_frame_time();
		let last_fps = gls.get_frames_per_second();
		let last_tps = gls.get_ticks_per_second();
		
		gls.next(|| {gfxroot.glfw.get_time()},
			|_now:f64| {
				router.borrow_mut().fire_event_at_lens("client", &mut TickEvent {});
			},
			
			|now: f64, interpolation: f32| {
				unsafe {
					gl::Clear(gl::COLOR_BUFFER_BIT
							| gl::DEPTH_BUFFER_BIT
							| gl::STENCIL_BUFFER_BIT
					);
				}
				
				let mut draw_event = DrawEvent {
					window_size, now, interpolation
				};
				router.borrow_mut().fire_event_at_lens("client", &mut draw_event);
				
				
				
				let (w, h) = gfxroot.window.get_framebuffer_size();
				render_state_gui.width = w;
				render_state_gui.height = h;
				
				render_state_gui.debug_text.clear();
				
				render_state_gui.debug_text.push((
					0.0, 16.0,
					format!("TCGE {version} \nFrametime: {mpf}ms \n{fps} FPS, {tps} TPS",
						version = env!("VERSION"),
						mpf = (frame_time * 1000.0).ceil(),
						fps = last_fps.floor(),
						tps = last_tps.round()
					)
				));
				
				match router.borrow_mut().nodes.get_node_component_downcast::<scene::Scene>(0) {
					Ok(scene) => {
						let camera = scene.camera.borrow();
						let position = camera.get_position(interpolation);
						
						render_state_gui.debug_text.push((
							0.0, (h as f32) - 2.0,
							format!("Camera: {x:.1}, {y:.1}, {z:.1} / {pitch:.0} {yaw:.0}",
								x = position.x,
								y = position.y,
								z = position.z,
								pitch = camera.rotation.x.round(),
								yaw = camera.rotation.y.round()
							)
						));
					},
					Err(_) => ()
				}
				
				
				render_gui(&mut render_state_gui);
			}
		);
		
		gfxroot.window.swap_buffers();
		gfxroot.glfw.poll_events();
	}
	
	Ok(())
}

struct GuiRenderState {
	width: i32, height: i32,
	ascii_renderer: render::text::AsciiTextRenderer,
	debug_text: Vec<(f32,f32,String)>
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
	
	render_state_gui.ascii_renderer.transform = projection;
	
	while let Some((x,y,text)) = render_state_gui.debug_text.pop() {
		render_state_gui.ascii_renderer.draw_text(text, 16.0, x, y);
	}
	
	render::utility::gl_pop_debug();
}

struct TickEvent {}
impl router::event::Event for TickEvent {
	fn is_passive(&self) -> bool {false}
}

struct DrawEvent {
	window_size: (i32, i32),
	now: f64,
	interpolation: f32,
}
impl router::event::Event for DrawEvent {
	fn is_passive(&self) -> bool {false}
}
