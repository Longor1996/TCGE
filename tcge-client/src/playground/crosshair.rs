use crate::render::*;
use std::rc::Rc;

pub struct CrosshairRenderer2D {
	gl: gl::Gl,
	material: Rc<materials::SolidColorMaterial>,
	mesh: VertexArray,
}

impl CrosshairRenderer2D {
	pub fn new(gl: &gl::Gl, shader: &Rc<materials::SolidColorMaterial>) -> Self {
		
		let mut mesh_2d = geometry::SimpleMeshBuilder::new();
		mesh_2d.push_vertex_with_uv(-1.0, -1.0, 0.0, 0.0, 0.0);
		mesh_2d.push_vertex_with_uv( 1.0, -1.0, 0.0, 1.0, 0.0);
		mesh_2d.push_vertex_with_uv(-1.0,  1.0, 0.0, 0.0, 1.0);
		mesh_2d.push_vertex_with_uv(-1.0,  1.0, 0.0, 0.0, 1.0);
		mesh_2d.push_vertex_with_uv( 1.0, -1.0, 0.0, 1.0, 0.0);
		mesh_2d.push_vertex_with_uv( 1.0,  1.0, 0.0, 1.0, 1.0);
		let mesh = mesh_2d.build(gl);
		
		Self {
			gl: gl.clone(),
			material: shader.clone(),
			mesh,
		}
	}
	
	pub fn draw(&self, projection: &nalgebra_glm::Mat4, width: i32, height: i32, size: f32) {
		self.gl.push_debug("Crosshair 2D");
		
		unsafe {
			self.gl.Enable(gl::BLEND);
			self.gl.BlendFunc(gl::ONE_MINUS_DST_COLOR, gl::ZERO);
		}
		
		let color = [1.0, 1.0, 1.0, 1.0];
		let scale = size / 2.0;
		
		let width = width as f32;
		let height = height as f32;
		
		let mut transform = nalgebra_glm::identity();
		
		transform = transform * nalgebra_glm::translation(&nalgebra_glm::Vec3::new (width/2.0, height/2.0, 0.0));
		transform = transform * nalgebra_glm::scaling(&nalgebra_glm::Vec3::new(scale, scale, 0.0));
		transform = projection * transform;
		
		self.material.shader.set_used();
		self.material.shader.set_uniform_matrix4(self.material.uniform_matrix, &transform);
		self.material.shader.set_uniform_vector4_raw(self.material.uniform_color, &color);
		self.mesh.draw_arrays(&self.gl);
		
		unsafe {
			self.gl.BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
		}
		
		self.gl.pop_debug();
	}
}

pub struct CrosshairRenderer3D {
	gl: gl::Gl,
	material: Rc<materials::SolidColorMaterial>,
	mesh: VertexArray,
}

impl CrosshairRenderer3D {
	
	pub fn new(gl: &gl::Gl, shader: &Rc<materials::SolidColorMaterial>) -> Self {
		let mesh = geometry::geometry_cube(gl, 0.5);
		
		Self {
			gl: gl.clone(),
			material: shader.clone(),
			mesh,
		}
	}
	
	pub fn draw(&self, camera: &nalgebra_glm::Mat4, pos: &blocks::BlockCoord) {
		self.gl.push_debug("Crosshair 3D");
		
		unsafe {
			self.gl.Enable(gl::BLEND);
			self.gl.Disable(gl::CULL_FACE);
			self.gl.BlendFunc(gl::ONE_MINUS_DST_COLOR, gl::ZERO);
			self.gl.PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
		}
		
		let color = nalgebra_glm::Vec4::new (1.0, 1.0, 1.0, 1.0);
		
		let mut transform = nalgebra_glm::identity();
		transform = transform * nalgebra_glm::translation(&nalgebra_glm::Vec3::new(0.5, 0.5, 0.5));
		transform = transform * nalgebra_glm::translation(&nalgebra_glm::Vec3::new(pos.x as f32, pos.y as f32, pos.z as f32));
		transform = transform * nalgebra_glm::scaling(&nalgebra_glm::vec3(1.02, 1.02, 1.02));
		transform = camera * transform;
		
		self.material.shader.set_used();
		self.material.shader.set_uniform_matrix4(self.material.uniform_matrix, &transform);
		self.material.shader.set_uniform_vector4(self.material.uniform_color, &color);
		self.mesh.draw_arrays(&self.gl);
		
		unsafe {
			self.gl.PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
			self.gl.BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
			self.gl.Enable(gl::CULL_FACE);
		}
		
		self.gl.pop_debug();
	}
	
}
