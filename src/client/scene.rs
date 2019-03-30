use super::super::router;
use super::render;
use super::geometry;
use super::freecam;
use resources::Resources;

pub struct Scene {
	pub camera: freecam::Camera,
	meshes: Vec<geometry::SimpleVao>,
	mesh_planequad: geometry::SimpleVao,
}

impl Scene {
	pub fn new() -> Scene {
		Scene {
			camera: freecam::Camera::new(),
			meshes: vec![
				geometry::geometry_test(),
				geometry::geometry_cube(1.0),
				// geometry::geometry_cube(-512.0),
			],
			mesh_planequad: geometry::geometry_planequad(1024.0),
		}
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

pub struct SceneRenderState {
	frame_id: i64,
	shader_grid: render::materials::ShaderGrid,
	shader_random: render::materials::ShaderRandom,
}

impl SceneRenderState {
	pub fn new(res: &Resources) -> Result<SceneRenderState, render::utility::Error> {
		let shader_grid = render::materials::ShaderGrid::new(&res)?;
		let shader_random = render::materials::ShaderRandom::new(&res)?;
		
		Ok(SceneRenderState {
			frame_id: 0,
			shader_grid,
			shader_random,
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

impl router::comp::Component for SceneRenderState {
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

pub fn render(render_state: &SceneRenderState, scene: &Scene, size: (i32, i32), now: f64, _interpolation:f32) {
	render::utility::gl_push_debug("Draw Scene");
	
	unsafe {
		gl::Enable(gl::DEPTH_TEST);
		gl::CullFace(gl::FRONT);
		gl::Enable(gl::CULL_FACE);
	}
	
	let camera = &scene.camera;
	
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
