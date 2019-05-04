//! Represents a simple prototypical 'game'-session.

use super::super::resources;
use super::super::router;
use super::render;
use super::geometry;
use super::freecam;
use super::blocks;

pub struct Scene {
	pub camera: freecam::Camera,
	meshes: Vec<geometry::SimpleMesh>,
	pub chunks: blocks::ChunkStorage,
}

impl Scene {
	pub fn new() -> Scene {
		let chunks = blocks::ChunkStorage::new();
		
		Scene {
			camera: freecam::Camera::new(),
			meshes: vec![
				// geometry::geometry_test(),
				// geometry::geometry_cube(1.0),
				// geometry::geometry_cube(-512.0),
			],
			chunks
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
	
	fn on_event(&mut self, _event: &mut router::event::Wrapper) {
		//
	}
}

pub struct SceneRenderer {
	frame_id: i64,
	grid: render::grid::Grid,
	shader_random: render::materials::ShaderRandom,
	crosshair_3d: render::crosshair::CrosshairRenderer3D,
	chunk_rmng: blocks::ChunkRenderManager,
}

impl SceneRenderer {
	pub fn new(res: &resources::Resources) -> Result<SceneRenderer, render::utility::Error> {
		let grid = render::grid::Grid::new(&res)?;
		let shader_random = render::materials::ShaderRandom::new(&res)?;
		let crosshair_3d = render::crosshair::CrosshairRenderer3D::new(&res)?;
		let chunk_rmng = blocks::ChunkRenderManager::new(res)?;
		
		Ok(SceneRenderer {
			frame_id: 0,
			grid,
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
	
	unsafe {
		gl::Enable(gl::DEPTH_TEST);
		gl::CullFace(gl::FRONT);
		gl::Enable(gl::CULL_FACE);
	}
	
	let camera = &scene.camera;
	
	let camera_position = camera.get_position(interpolation);
	let camera_transform = camera.transform(size, interpolation, true);
	
	render_state.grid.draw(&camera_transform, &camera_position);
	
	let shader_random = &render_state.shader_random;
	shader_random.shader_program.set_used();
	shader_random.shader_program.uniform_matrix4(shader_random.uniform_matrix, camera_transform);
	shader_random.shader_program.uniform_scalar(shader_random.uniform_time, now as f32);
	
	for mesh in scene.meshes.iter() {
		mesh.draw(gl::TRIANGLES);
	}
	
	// Render chunks!
	render_state.chunk_rmng.render(scene, camera_transform);
	
	if let Some(target) = &scene.camera.target {
		render_state.crosshair_3d.draw(camera_transform, target);
	}
	
	render::utility::gl_pop_debug();
}
