use super::super::super::resources;
use super::super::geometry;
use super::cgmath::Matrix4;
use super::cgmath::Vector3;
use super::materials::ShaderGrid;

pub struct Grid {
	size: f32,
	mesh: geometry::SimpleVao,
	shader: ShaderGrid,
}

impl Grid {
	pub fn new(res: &resources::Resources) -> Result<Grid, super::utility::Error> {
		let shader = ShaderGrid::new(&res)?;
		let mesh = geometry::geometry_planequad(2000.0);
		
		Ok(Grid {
			size: 100.0,
			mesh,
			shader,
		})
	}
	
	pub fn draw(&self, camera_transform: &Matrix4<f32>, camera_position: &Vector3<f32>) {
		super::utility::gl_push_debug("Draw Grid");
		
		unsafe {
			gl::Enable(gl::BLEND);
			gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
			gl::Disable(gl::DEPTH_TEST);
		}
		
		let position = camera_position;
		
		let grid_transform = Matrix4::from_translation(Vector3::<f32> {
			x: (position.x / self.size).round() * self.size,
			y: 0.0,
			z: (position.z / self.size).round() * self.size,
		});
		
		let transform = camera_transform * grid_transform;
		
		let shader_grid = &self.shader;
		shader_grid.shader_program.set_used();
		shader_grid.shader_program.uniform_matrix4(shader_grid.uniform_matrix, transform);
		self.mesh.draw(gl::TRIANGLES);
		unsafe {
			gl::Enable(gl::DEPTH_TEST);
			gl::Disable(gl::BLEND);
		}
		
		super::utility::gl_pop_debug();
	}
}