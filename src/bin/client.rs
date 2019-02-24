use std::rc::Rc;
use std::cell::Ref;
use std::cell::RefCell;

extern crate failure;
use failure::Fail;
extern crate time;
extern crate cgmath;

extern crate glfw;
use self::glfw::{Context, Key, Action};
extern crate gl;

extern crate TCGE;
use TCGE::resources::Resources;
use TCGE::blocks::universe;
use TCGE::client::render_gl;
use TCGE::gameloop;

fn main() {
	println!("Hello, Client!");
	println!("Version: {}", env!("VERSION"));
	
	if let Err(e) = run() {
		use std::fmt::Write;
		let mut result = String::new();
		
		for (i, cause) in e.causes().collect::<Vec<_>>().into_iter().enumerate() {
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
		
		println!("{}", result);
	}
	
	println!("Goodbye!");
}

fn run() -> Result<(), failure::Error> {
	// ------------------------------------------
	let res = Resources::from_exe_path()?;
	
	// ------------------------------------------
	let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS)?;
	
	glfw.window_hint(glfw::WindowHint::ContextVersion(3,2));
	glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
	
	#[cfg(target_os = "macos")]
		glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));
	glfw.window_hint(glfw::WindowHint::OpenGlDebugContext(true));
	glfw.window_hint(glfw::WindowHint::Samples(Some(4)));
	
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
	
	// ------------------------------------------
	gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);
	
	/*
	unsafe {
		let depth_bits = glfw::ffi::glfwGetWindowAttrib(window.window_ptr(), glfw::ffi::DEPTH_BITS);
		let depth_bits = gl::GetFramebufferAttachmentParameteriv(gl::FRAMEBUFFER, gl::GL_DEPTH_ATTACHMENT);
		println!("Available depth bits: {}", depth_bits);
	}
	*/
	
	// ------------------------------------------
	let shader_grid = ShaderGrid::new(&res)?;
	let shader_random = ShaderRandom::new(&res)?;
	let shader_solid_color = ShaderSolidColor::new(&res)?;
	
	// ------------------------------------------
	let mut render_state = RenderState {
		frame_id: 0,
		shader_grid,
		shader_random,
		shader_solid_color
	};
	
	let mut cursor = Cursor {pos_x: 0.0, pos_y: 0.0, mov_x: 0.0, mov_y: 0.0};
	
	let local_universe = universe::define_universe();
	
	let scene = Rc::new(RefCell::new(Option::Some(Scene {
		camera: Camera {
			position: cgmath::Vector3 {x: 0.0, y: 1.8, z: 0.0},
			velocity: cgmath::Vector3 {x: 0.0, y: 0.0, z: 0.0},
			rotation: cgmath::Vector2 {x: 0.0, y: 90.0},
			position_last: cgmath::Vector3 {x: 0.0, y: 1.8, z: 0.0},
			velocity_last: cgmath::Vector3 {x: 0.0, y: 0.0, z: 0.0},
			rotation_last: cgmath::Vector2 {x: 0.0, y: 90.0}
		},
		meshes: vec![geometry_test()],
		mesh_grid: geometry_grid(),
		mesh_planequad: geometry_planequad(10.0),
		local_universe: local_universe
	})));
	
	// ------------------------------------------
	let mut gls = gameloop::new_gameloop(30);
	
	while !window.should_close() {
		process_events(
			&mut window,
			&events,
			&mut cursor,
			&mut *scene.borrow_mut()
		);
		
		let window_size = window.get_framebuffer_size();
		let mut reset_render_state = false;
		
		gameloop::gameloop_next(&mut gls,
			|| {glfw.get_time()},
			
			|_now:f64| {
				// println!("It is now {}", now);
				
				scene.borrow().as_ref().map(|scene| {
					println!("{}", scene.camera);
				});
				
				scene.borrow_mut().as_mut().map(|mut_scene| {
					mut_scene.camera.update_movement(&window);
				});
				
				reset_render_state = true;
			},
			
			|now: f64, interpolation: f32| {
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

use std::sync::mpsc::Receiver;
fn process_events(
	window: &mut glfw::Window,
	events: &Receiver<(f64, glfw::WindowEvent)>,
	cursor: &mut Cursor,
	opt_scene: &mut Option<Scene>
) {
	for(_, event) in glfw::flush_messages(events) {
		match event {
			glfw::WindowEvent::FramebufferSize(width, height) => {
				unsafe {gl::Viewport(0, 0, width, height)}
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
	camera: Camera,
	meshes: Vec<SimpleVAO>,
	mesh_grid: SimpleVAO,
	mesh_planequad: SimpleVAO,
	local_universe: universe::BlockUniverse,
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
	shader_grid: ShaderGrid,
	shader_random: ShaderRandom,
	shader_solid_color: ShaderSolidColor,
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

struct ShaderRandom {
	shader_program: render_gl::Program,
	uniform_matrix: i32,
	uniform_time: i32,
}
impl ShaderRandom {
	fn new(res: &Resources) -> Result<ShaderRandom, TCGE::client::render_gl::Error> {
		let shader_program = render_gl::Program::from_res(&res, "shaders/triangle")?;
		let uniform_matrix = shader_program.uniform_location("transform");
		let uniform_time = shader_program.uniform_location("time");
		Ok(ShaderRandom {
			shader_program,
			uniform_matrix,
			uniform_time
		})
	}
}

struct ShaderSolidColor {
	shader_program: render_gl::Program,
	uniform_matrix: i32,
	uniform_color: i32,
}
impl ShaderSolidColor {
	fn new(res: &Resources) -> Result<ShaderSolidColor, TCGE::client::render_gl::Error> {
		let shader_program = render_gl::Program::from_res(&res, "shaders/solid-color")?;
		let uniform_matrix = shader_program.uniform_location("transform");
		let uniform_color = shader_program.uniform_location("color");
		Ok(ShaderSolidColor {
			shader_program,
			uniform_matrix,
			uniform_color
		})
	}
}

struct ShaderGrid {
	shader_program: render_gl::Program,
	uniform_matrix: i32
}
impl ShaderGrid {
	fn new(res: &Resources) -> Result<ShaderGrid, TCGE::client::render_gl::Error> {
		let shader_program = render_gl::Program::from_res(&res, "shaders/grid")?;
		let uniform_matrix = shader_program.uniform_location("transform");
		Ok(ShaderGrid {
			shader_program,
			uniform_matrix
		})
	}
}

fn render(render_state: &RenderState, scene: &Scene, camera: &Camera, size: (i32, i32), now: f64, _interpolation:f32) {
	unsafe {
		gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
		gl::Enable(gl::DEPTH_TEST);
		gl::CullFace(gl::FRONT);
		gl::Enable(gl::CULL_FACE);
	}
	
	println!("Render Frame [ id: {}, size: {} x {}, time: {}, delta: {}]",
		render_state.frame_id, size.0, size.1, now, _interpolation
	);
	
	let camera_transform = camera.transform(size, _interpolation, true);
	
	/*
	let shader_solid_color = &render_state.shader_solid_color;
	let grid_color = cgmath::Vector4 {x: 1.0, y: 1.0, z: 1.0, w: 1.0};
	shader_solid_color.shader_program.set_used();
	shader_solid_color.shader_program.uniform_matrix4(shader_solid_color.uniform_matrix, camera_transform);
	shader_solid_color.shader_program.uniform_vector4(shader_solid_color.uniform_color, grid_color);
	scene.mesh_grid.draw(gl::LINES);
	*/
	
	unsafe {
		gl::Enable(gl::BLEND);
		gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
	}
	let shader_grid = &render_state.shader_grid;
	shader_grid.shader_program.set_used();
	shader_grid.shader_program.uniform_matrix4(shader_grid.uniform_matrix, camera_transform);
	scene.mesh_planequad.draw(gl::TRIANGLES);
	unsafe {gl::Disable(gl::BLEND) }
	
	let shader_random = &render_state.shader_random;
	shader_random.shader_program.set_used();
	shader_random.shader_program.uniform_matrix4(shader_random.uniform_matrix, camera_transform);
	shader_random.shader_program.uniform_scalar(shader_random.uniform_time, now as f32);
	
	for mesh in scene.meshes.iter() {
		mesh.draw(gl::TRIANGLES);
	}
}

struct SimpleVAO {
	handle: gl::types::GLuint,
	count: i32,
}
impl SimpleVAO {
	fn draw(&self, mode: u32) {
		unsafe {
			gl::BindVertexArray(self.handle);
			gl::DrawArrays(mode, 0, self.count);
		}
	}
}

fn geometry_planequad(s: f32) -> SimpleVAO {
	let vertices: Vec<f32> = vec![
		-s, 0.0,  s,
		 s, 0.0,  s,
		-s, 0.0, -s,
		 s, 0.0,  s,
		 s, 0.0, -s,
		-s, 0.0, -s
	];
	
	let mut vbo: gl::types::GLuint = 0;
	
	unsafe {
		gl::GenBuffers(1, &mut vbo);
	}
	
	unsafe {
		gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
		gl::BufferData(
			gl::ARRAY_BUFFER,
			(vertices.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
			vertices.as_ptr() as *const gl::types::GLvoid,
			gl::STATIC_DRAW
		);
		gl::BindBuffer(gl::ARRAY_BUFFER, 0);
	}
	
	let mut vao: gl::types::GLuint = 0;
	unsafe {
		gl::GenVertexArrays(1, &mut vao);
		gl::BindVertexArray(vao);
		gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
		gl::EnableVertexAttribArray(0);
		gl::VertexAttribPointer(
			0,
			3,
			gl::FLOAT, gl::FALSE,
			(3 * std::mem::size_of::<f32>()) as gl::types::GLint,
			std::ptr::null()
		);
		gl::BindBuffer(gl::ARRAY_BUFFER, 0);
		gl::BindVertexArray(0);
	}
	
	SimpleVAO {
		handle: vao,
		count: (vertices.len()/3) as i32
	}
}

fn geometry_grid() -> SimpleVAO {
	let mut vertices: Vec<f32> = vec![];
	
	let range: i32 = 256;
	let size: f32 = range as f32;
	
	for x in -range .. range {
		vertices.extend(&vec![
			-size, 0.0, x as f32,
			 size, 0.0, x as f32
		]);
		vertices.extend(&vec![
			x as f32, 0.0, -size,
			x as f32, 0.0, size
		]);
	}
	
	let mut vbo: gl::types::GLuint = 0;
	
	unsafe {
		gl::GenBuffers(1, &mut vbo);
	}
	
	unsafe {
		gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
		gl::BufferData(
			gl::ARRAY_BUFFER,
			(vertices.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
			vertices.as_ptr() as *const gl::types::GLvoid,
			gl::STATIC_DRAW
		);
		gl::BindBuffer(gl::ARRAY_BUFFER, 0);
	}
	
	let mut vao: gl::types::GLuint = 0;
	unsafe {
		gl::GenVertexArrays(1, &mut vao);
		gl::BindVertexArray(vao);
		gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
		gl::EnableVertexAttribArray(0);
		gl::VertexAttribPointer(
			0,
			3,
			gl::FLOAT, gl::FALSE,
			(3 * std::mem::size_of::<f32>()) as gl::types::GLint,
			std::ptr::null()
		);
		gl::BindBuffer(gl::ARRAY_BUFFER, 0);
		gl::BindVertexArray(0);
	}
	
	SimpleVAO {
		handle: vao,
		count: (vertices.len()/2) as i32
	}
}

fn geometry_test() -> SimpleVAO {
	let mut vertices: Vec<f32> = vec![
		-0.5, -0.5, -10.0,
		0.5, -0.5, -10.0,
		0.0, 0.5, -10.0
	];
	
	vertices.extend(&vec![
		-20.0, 0.0, -20.0,
		0.0, 0.0,  20.0,
		20.0, 0.0, -20.0
	]);
	
	vertices.extend(&vec![
		-5.0, 0.0, 30.0,
		0.0, 9.0, 30.0,
		5.0, 0.0, 30.0
	]);
	
	let mut vbo: gl::types::GLuint = 0;
	unsafe {
		gl::GenBuffers(1, &mut vbo);
	}
	unsafe {
		gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
		gl::BufferData(
			gl::ARRAY_BUFFER,
			(vertices.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
			vertices.as_ptr() as *const gl::types::GLvoid,
			gl::STATIC_DRAW
		);
		gl::BindBuffer(gl::ARRAY_BUFFER, 0);
	}
	
	let mut vao: gl::types::GLuint = 0;
	unsafe {
		gl::GenVertexArrays(1, &mut vao);
		gl::BindVertexArray(vao);
		gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
		gl::EnableVertexAttribArray(0);
		gl::VertexAttribPointer(
			0,
			3,
			gl::FLOAT, gl::FALSE,
			(3 * std::mem::size_of::<f32>()) as gl::types::GLint,
			std::ptr::null()
		);
		gl::BindBuffer(gl::ARRAY_BUFFER, 0);
		gl::BindVertexArray(0);
	}
	
	SimpleVAO {
		handle: vao,
		count: (vertices.len()/3) as i32
	}
}

// TODO: Camera needs PlayerController/ClientInput...
#[derive(Debug)]
struct Camera {
	position: cgmath::Vector3<f32>,
	velocity: cgmath::Vector3<f32>,
	rotation: cgmath::Vector2<f32>,
	position_last: cgmath::Vector3<f32>,
	velocity_last: cgmath::Vector3<f32>,
	rotation_last: cgmath::Vector2<f32>,
}

impl Camera {
	fn transform(&self, size: (i32,i32), interpolation: f32, translation: bool ) -> cgmath::Matrix4<f32> {
		use cgmath::Matrix4;
		use cgmath::InnerSpace;
		use cgmath::ElementWise;
		
		let (width, height) = size;
		let fov = cgmath::Rad::from(cgmath::Deg(90.0));
		
		let perspective = cgmath::PerspectiveFov {
			fovy: fov,
			aspect: width as f32 / height as f32,
			near: 0.1, far: 1024.0
		};
		
		let perspective = Matrix4::from(perspective);
		
		// this next section can most certainly be written with less code...
		let mut camera = Matrix4::new(
			1.0, 0.0, 0.0, 0.0,
			0.0, 1.0, 0.0, 0.0,
			0.0, 0.0, 1.0, 0.0,
			0.0, 0.0, 0.0, 1.0
		);
		
		let pitch = cgmath::Deg(self.rotation.x);
		let yaw = cgmath::Deg(self.rotation.y);
		
		camera = camera * Matrix4::from_angle_x(pitch);
		camera = camera * Matrix4::from_angle_y(yaw);
		camera = camera * Matrix4::from_nonuniform_scale(1.0,1.0,-1.0);
		
		if translation {
			// simple movement prediction formula
			let pos = self.position + (self.velocity * interpolation);
			camera = camera * Matrix4::from_translation(-pos);
		}
		
		// return multiplied matrix
		perspective * camera
	}
	
	fn update_rotation(&mut self, yaw: f32, pitch: f32) {
		self.rotation_last.clone_from(& self.rotation);
		
		let mouse_sensivity = 0.5;
		
		self.rotation.x += pitch * mouse_sensivity;
		self.rotation.x = clamp(self.rotation.x, -90.0, 90.0);
		
		self.rotation.y += yaw * mouse_sensivity;
		self.rotation.y = wrap(self.rotation.y , 360.0);
	}
	
	fn update_movement(&mut self, window: & glfw::Window) {
		use cgmath::Vector3;
		use cgmath::Matrix4;
		use cgmath::Transform;
		
		self.position_last.clone_from(& self.position);
		self.velocity_last.clone_from(& self.velocity);
		
		let mut move_speed = 0.5;
		
		if window.get_key(Key::LeftShift) == Action::Press {
			move_speed = move_speed * 4.0;
		}
		
		if window.get_key(Key::LeftControl) == Action::Press {
			self.position += Vector3::new(0.0, -1.0, 0.0) * move_speed;
		}
		if window.get_key(Key::Space) == Action::Press {
			self.position += Vector3::new(0.0, 1.0, 0.0) * move_speed;
		}
		
		let yaw = cgmath::Deg(self.rotation.y);
		let mat = Matrix4::from_angle_y(yaw);
		
		let forward = Vector3::new(0.0, 0.0, 1.0);
		let forward = Matrix4::transform_vector(&mat, forward);
		if window.get_key(Key::W) == Action::Press {
			self.position += forward * move_speed;
		}
		
		let backward = Vector3::new(0.0, 0.0, -1.0);
		let backward = Matrix4::transform_vector(&mat, backward);
		if window.get_key(Key::S) == Action::Press {
			self.position += backward * move_speed;
		}
		
		let left = Vector3::new(-1.0, 0.0, 0.0);
		let left = Matrix4::transform_vector(&mat, left);
		if window.get_key(Key::A) == Action::Press {
			self.position += left * move_speed;
		}
		
		let right = Vector3::new(1.0, 0.0, 0.0);
		let right = Matrix4::transform_vector(&mat, right);
		if window.get_key(Key::D) == Action::Press {
			self.position += right * move_speed;
		}
	}
}

impl std::fmt::Display for Camera {
	fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(fmt, "Camera [x: {}, z: {}, pitch: {}, yaw: {} ] & LastCamera [x: {}, z: {}, pitch: {}, yaw: {} ]",
			   self.position.x,
			   self.position.z,
			   self.rotation.x,
			   self.rotation.y,
			   self.position_last.x,
			   self.position_last.z,
			   self.rotation_last.x,
			   self.rotation_last.y
		)
	}
}

fn clamp(x: f32, min: f32, max: f32) -> f32 {
	if x < min { return min; }
	if x > max { return max; }
	x
}

fn wrap(mut x: f32, r: f32) -> f32 {
	while x < 0.0 {
		x += r;
	}
	while x > r {
		x -= r;
	}
	
	x
}