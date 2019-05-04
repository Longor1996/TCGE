//! This is the application entry-point and mainloop for the client.
//!
//! All related code not contained in this file resides in the [`client`-crate](../tcge/client/index.html).

use std::rc::Rc;
use std::cell::{RefCell};
use core::borrow::Borrow;

#[macro_use]
extern crate log;
extern crate simplelog;

extern crate failure;
#[allow(unused_imports)]
use failure::Fail;

extern crate gl;

extern crate tcge;
use tcge::resources;
use tcge::router;
use tcge::util::gameloop;
use tcge::client;
use tcge::client::cmd_opts;
use tcge::client::context;
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
		
		event.downcast::<router::lens::MoveCompletionEvent>().map(|move_end| {
			match move_end {
				router::lens::MoveCompletionEvent::Finished => info!("Reached destination node."),
				router::lens::MoveCompletionEvent::Aborted => error!("Failed to reach destination node."),
			}
		});
		
		event.downcast::<client::TickEvent>().map(|_tick| {
			let scene = context.get_mut_component_downcast::<scene::Scene>();
			let gfx = context.get_mut_component_downcast::<context::GlfwContextComponent>();
			
			match scene {
				Ok(scene) => {
					match gfx {
						Ok(gfx) => {
							scene.camera.update_movement(gfx.window.borrow());
							scene.update_targeted_block();
						},
						Err(_) => ()
					}
				}
				Err(_) => ()
			}
			
			match context.get_mut_component_downcast::<scene::SceneRenderer>() {
				Ok(scene_render_state) => {
					scene_render_state.reset();
				},
				Err(_) => ()
			};
		});
		
		event.downcast::<client::DrawEvent>().map(|draw| {
			let scene = context.get_mut_component_downcast::<scene::Scene>();
			let scene_renderer = context.get_mut_component_downcast::<scene::SceneRenderer>();
			
			if scene.is_err() {
				panic!("This ain't supposed to happen!");
			}
			
			match scene {
				Ok(scene) => {
					match scene_renderer {
						Ok(scene_renderer) => {
							scene_renderer.begin();
							scene::render(
								scene_renderer,
								scene,
								draw.window_size,
								draw.now,
								draw.interpolation
							);
							scene_renderer.end();
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
	let gfx = context::GlfwContextComponent::new(&opts)?;
	
	// Give the router ownership of the Graphics-Context... then sneakily grab it back!
	// This is the **only** place in the code where it's okay to do this.
	router.nodes.set_node_component(0, Box::new(gfx))?;
	let gfx = router.nodes
		.get_mut_node_component_downcast::<context::GlfwContextComponent>(0)?;
	
	// ------------------------------------------
	
	let ascii_renderer = render::text::AsciiTextRenderer::load(&res, "hack")?;
	let mut render_state_gui = GuiRenderState {
		width: 0.0, height: 0.0,
		ascii_renderer,
		crosshair_2d: render::crosshair::CrosshairRenderer2D::new(&res)?,
		debug_text: vec![],
	};
	
	// ------------------------------------------
	
	info!("Initializing scene...");
	
	let mut scene = scene::Scene::new();
	scene.camera.active = gfx.window.get_cursor_mode() == glfw::CursorMode::Disabled;
	router.nodes.set_node_component(0, Box::new(scene))?;
	
	let scene_renderer = scene::SceneRenderer::new(&res)?;
	router.nodes.set_node_component(0, Box::new(scene_renderer))?;
	
	// ------------------------------------------
	
	// Create the client lens...
	router.new_lens("client", &|lens| {
		info!("Initial route: {}", opts.path);
		lens.state = router::lens::State::Moving(opts.path.clone(), 0);
		
		Some(Box::new(ClientLens {
			// nothing here yet
		}))
	});
	
	// Then put the router into a RefCell and (hopefully) never touch it again!
	let router = Rc::new(RefCell::new(router));
	
	// ------------------------------------------
	info!("Initializing and starting gameloop...");
	let mut gameloop = gameloop::GameloopState::new(30, true);
	
	while !router.borrow_mut().update() && !gfx.window.should_close() {
		gfx.process_events(&mut router.borrow_mut());
		
		let window_size = gfx.window.get_framebuffer_size();
		let frame_time  = gameloop.get_frame_time();
		let last_fps = gameloop.get_frames_per_second();
		let last_tps = gameloop.get_ticks_per_second();
		
		gameloop.next(|| {gfx.glfw.get_time()},
			|_now:f64| {
				router.borrow_mut().fire_event_at_lens("client", &mut client::TickEvent {});
			},
			
			|now: f64, interpolation: f32| {
				unsafe {
					gl::Clear(gl::COLOR_BUFFER_BIT
							| gl::DEPTH_BUFFER_BIT
							| gl::STENCIL_BUFFER_BIT
					);
				}
				
				let mut draw_event = client::DrawEvent {
					window_size, now, interpolation
				};
				router.borrow_mut().fire_event_at_lens("client", &mut draw_event);
				
				let (w, h) = gfx.window.get_framebuffer_size();
				render_state_gui.width = w as f32;
				render_state_gui.height = h as f32;
				
				render_state_gui.debug_text.clear();
				
				render_state_gui.debug_text.push((
					0.0, 0.0,
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
						let rotation = camera.get_rotation(interpolation);
						
						render_state_gui.debug_text.push((
							0.0, (h as f32) - 16.0 -  2.0,
							format!("Camera: {x:.1}, {y:.1}, {z:.1} / {pitch:.0} {yaw:.0}",
								x = position.x,
								y = position.y,
								z = position.z,
								pitch = rotation.x.round(),
								yaw   = rotation.y.round()
							)
						));
					},
					Err(_) => ()
				}
				
				render_gui(&mut render_state_gui);
			}
		);
		
		gfx.swap_and_poll();
	}
	
	Ok(())
}

struct GuiRenderState {
	width: f32, height: f32,
	ascii_renderer: render::text::AsciiTextRenderer,
	crosshair_2d: render::crosshair::CrosshairRenderer2D,
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
	
	let width = render_state_gui.width;
	let height = render_state_gui.height;
	
	let projection = cgmath::ortho(
		0.0,
		width,
		height,
		0.0,
		-1.0,1.0
	);
	
	render_state_gui.crosshair_2d.draw(projection, width, height, 4.0);
	
	render_state_gui.ascii_renderer.transform = projection;
	
	while let Some((x,y,text)) = render_state_gui.debug_text.pop() {
		render_state_gui.ascii_renderer.draw_text(&text, 16.0, x, y);
	}
	
	render::utility::gl_pop_debug();
}
