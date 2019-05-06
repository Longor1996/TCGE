//! Represents a simple prototypical 'game'-session.

use super::super::resources;
use super::super::router;
use super::render;
use super::geometry;
use super::freecam;
use super::blocks;
use super::super::blocks as blockdef;

pub struct Scene {
	pub camera: freecam::Camera,
	meshes: Vec<geometry::SimpleMesh>,
	pub blockdef: blockdef::UniverseRef,
	pub chunks: blocks::ChunkStorage,
}

impl Scene {
	pub fn new() -> Scene {
		let config = Scene::load_config().expect("Failed to load scene config.");
		
		let blockdef = blockdef::universe::define_universe(&config);
		let chunks = blocks::ChunkStorage::new(blockdef.clone(), &config);
		
		Scene {
			camera: freecam::Camera::new(),
			meshes: vec![
				// geometry::geometry_test(),
				// geometry::geometry_cube(1.0),
				// geometry::geometry_cube(-512.0),
			],
			blockdef,
			chunks,
		}
	}
	
	pub fn load_config() -> Option<toml::value::Table> {
		let exe_file_name = ::std::env::current_exe().ok()?;
		let exe_path = exe_file_name.parent()?;
		let config_dir = exe_path.join("config");
		let config_file = config_dir.join("test-scene.toml");
		let mut config_file = std::fs::File::open(config_file.as_path()).ok()?;
		
		use std::io::Read;
		let mut config_str = String::new();
		config_file.read_to_string(&mut config_str).ok()?;
		
		let config = config_str.parse::<toml::Value>().ok()?;
		if let Some(config) = config.as_table() {
			return Some(config.clone());
		} else {
			return None;
		}
	}
	
	pub fn update_targeted_block(&mut self) {
		let src = self.camera.get_position(1.0);
		let dir = self.camera.get_look_dir(1.0);
		let len = 16.0;
		
		use super::blocks;
		let mut rc = blocks::BlockRaycast::new_from_src_dir_len(src, dir, len);
		
		let target = match self.chunks.raycast(&mut rc) {
			Some((_, curr_pos, _)) => {
				Some(curr_pos)
			},
			None => None
		};
		
		self.camera.target = target;
	}
	
}

impl router::comp::Component for Scene {
	fn get_type_name(&self) -> &'static str {
		"Scene"
	}
	
	fn on_attachment(&mut self, _node_id: usize) {}
	fn on_detachment(&mut self, _node_id: usize) {}
	
	fn on_load(&mut self) {}
	fn on_unload(&mut self) {}
	
	fn on_event(&mut self, event: &mut router::event::Wrapper) {
		
		if let Some(event) = event.downcast::<super::settings::SettingsReloadEvent>() {
			self.camera.apply_settings(event.settings);
		}
		
	}
}

pub struct SceneRenderer {
	frame_id: i64,
	grid: render::grid::Grid,
	sky_renderer: SkyRenderer,
	shader_random: render::materials::ShaderRandom,
	crosshair_3d: render::crosshair::CrosshairRenderer3D,
	chunk_rmng: blocks::ChunkRenderManager,
}

impl SceneRenderer {
	pub fn new(res: &resources::Resources, scene: &Scene) -> Result<SceneRenderer, render::utility::Error> {
		let grid = render::grid::Grid::new(res)?;
		let sky_renderer = SkyRenderer::new(res)?;
		let shader_random = render::materials::ShaderRandom::new(res)?;
		let crosshair_3d = render::crosshair::CrosshairRenderer3D::new(res)?;
		let chunk_rmng = blocks::ChunkRenderManager::new(res, scene.blockdef.clone())?;
		
		Ok(SceneRenderer {
			frame_id: 0,
			grid,
			sky_renderer,
			crosshair_3d,
			shader_random,
			chunk_rmng,
		})
	}
	
	pub fn begin(&mut self) {
		self.frame_id = self.frame_id + 1;
	}
	
	pub fn end(&mut self) {
		// ...?
	}
	
	pub fn reset(&mut self) {
		self.frame_id = 0;
	}
}

impl router::comp::Component for SceneRenderer {
	fn get_type_name(&self) -> &'static str {
		"SceneRenderState"
	}
	
	fn on_attachment(&mut self, _node_id: usize) {}
	fn on_detachment(&mut self, _node_id: usize) {}
	
	fn on_load(&mut self) {}
	fn on_unload(&mut self) {}
	
	fn on_event(&mut self, _event: &mut router::event::Wrapper) {
		//
	}
}

pub fn render(render_state: &mut SceneRenderer, scene: &Scene, size: (i32, i32), now: f64, interpolation:f32) {
	render::utility::gl_push_debug("Draw Scene");
	
	render_state.sky_renderer.render(&scene.camera, size, now, interpolation);
	
	unsafe {
		gl::Enable(gl::DEPTH_TEST);
		gl::CullFace(gl::FRONT);
		gl::Enable(gl::CULL_FACE);
	}
	
	let camera = &scene.camera;
	
	let camera_position = camera.get_position(interpolation);
	let camera_proj = camera.projection(interpolation, size);
	let camera_view = camera.transform(interpolation, true);
	let camera_matrix = camera_proj * camera_view;
	
	render_state.grid.draw(&camera_matrix, &camera_position);
	
	let shader_random = &render_state.shader_random;
	shader_random.shader_program.set_used();
	shader_random.shader_program.uniform_matrix4(shader_random.uniform_matrix, camera_matrix);
	shader_random.shader_program.uniform_scalar(shader_random.uniform_time, now as f32);
	
	for mesh in scene.meshes.iter() {
		mesh.draw(gl::TRIANGLES);
	}
	
	// Render chunks!
	render_state.chunk_rmng.render(scene, camera_matrix);
	
	if let Some(target) = &scene.camera.target {
		render_state.crosshair_3d.draw(camera_matrix, target);
	}
	
	render::utility::gl_pop_debug();
}

struct SkyRenderer {
	skybox: render::geometry::SimpleMesh,
	shader: SkyShader,
}

impl SkyRenderer {
	fn new(res: &resources::Resources) -> Result<Self, render::utility::Error> {
		let skybox = render::geometry::geometry_cube(10.0);
		let shader = SkyShader::new(res)?;
		
		Ok(SkyRenderer {
			skybox,
			shader,
		})
	}
	
	fn render(&mut self, camera: &freecam::Camera, size: (i32, i32), _now: f64, interpolation:f32) {
		unsafe {
			gl::Disable(gl::DEPTH_TEST);
			gl::Disable(gl::CULL_FACE);
		}
		
		let camera_proj = camera.projection(interpolation, size);
		let camera_view = camera.transform(interpolation, false);
		let transform = camera_proj * camera_view;
		
		let position = camera.get_position(interpolation);
		let color = cgmath::Vector4 {x: 0.3, y: 0.6, z: 1.0, w: 1.0};
		
		self.shader.shader_program.set_used();
		self.shader.shader_program.uniform_matrix4(self.shader.uniform_matrix, transform);
		self.shader.shader_program.uniform_vector3(self.shader.uniform_camera, position);
		self.shader.shader_program.uniform_vector4(self.shader.uniform_color, color);
		
		self.skybox.draw(gl::TRIANGLES);
		
		unsafe {
			gl::Enable(gl::DEPTH_TEST);
			gl::Enable(gl::CULL_FACE);
		}
	}
}

pub struct SkyShader {
	pub shader_program: render::utility::Program,
	pub uniform_matrix: i32,
	pub uniform_camera: i32,
	pub uniform_color: i32,
}

impl SkyShader {
	pub fn new(res: &resources::Resources) -> Result<Self, render::utility::Error> {
		let shader_program = render::utility::Program::from_res(&res, "shaders/sky")?;
		let uniform_matrix = shader_program.uniform_location("transform");
		let uniform_camera = shader_program.uniform_location("camera");
		let uniform_color = shader_program.uniform_location("color");
		Ok(SkyShader {
			shader_program,
			uniform_matrix,
			uniform_camera,
			uniform_color
		})
	}
}