use super::utility;
use super::geometry;
use super::materials;
use super::super::super::resources;
use cgmath::Matrix4;

pub struct CrosshairRenderer {
	mesh: geometry::SimpleMesh,
	shader: materials::ShaderSolidColor,
}

impl CrosshairRenderer {
	
	pub fn new(res: &resources::Resources) -> Result<CrosshairRenderer, utility::Error> {
		let shader = materials::ShaderSolidColor::new(res)?;
		
		let mut mesh = geometry::SimpleMeshBuilder::new();
		mesh.push_vertex_with_uv(-1.0, -1.0, 0.0, 0.0, 0.0);
		mesh.push_vertex_with_uv( 1.0, -1.0, 0.0, 1.0, 0.0);
		mesh.push_vertex_with_uv(-1.0,  1.0, 0.0, 0.0, 1.0);
		
		mesh.push_vertex_with_uv(-1.0,  1.0, 0.0, 0.0, 1.0);
		mesh.push_vertex_with_uv( 1.0, -1.0, 0.0, 1.0, 0.0);
		mesh.push_vertex_with_uv( 1.0,  1.0, 0.0, 1.0, 1.0);
		let mesh = mesh.build();
		
		Ok(CrosshairRenderer {
			mesh,
			shader
		})
	}
	
	pub fn draw(&self, projection: cgmath::Matrix4<f32>, width: f32, height: f32, size: f32) {
		utility::gl_push_debug("Crosshair");
		
		unsafe {
			gl::Enable(gl::BLEND);
			gl::Enable(gl::TEXTURE_2D);
			gl::BlendFunc(gl::ONE_MINUS_DST_COLOR, gl::ZERO);
		}
		
		let color = cgmath::Vector4::<f32> {x: 1.0, y: 1.0, z: 1.0, w: 1.0};
		let scale = size / 2.0;
		
		let mut transform = cgmath::One::one();
		transform = transform * Matrix4::from_translation(cgmath::Vector3::<f32> {x: width/2.0, y: height/2.0, z: 0.0});
		transform = transform * Matrix4::from_nonuniform_scale(scale, scale, 0.0);
		transform = projection * transform;
		
		self.shader.shader_program.set_used();
		self.shader.shader_program.uniform_matrix4(self.shader.uniform_matrix, transform);
		self.shader.shader_program.uniform_vector4(self.shader.uniform_color, color);
		self.mesh.draw(gl::TRIANGLES);
		
		unsafe {
			gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
			gl::Disable(gl::TEXTURE_2D);
		}
		
		utility::gl_pop_debug();
	}
	
}