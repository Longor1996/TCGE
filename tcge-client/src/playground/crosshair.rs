use cgmath::Matrix4;
use crate::render::*;
use std::rc::Rc;

const HALF_VEC: cgmath::Vector3<f32> = cgmath::Vector3::<f32> {x: 0.5, y: 0.5, z: 0.5};

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
	
	pub fn draw(&self, projection: &cgmath::Matrix4<f32>, width: i32, height: i32, size: f32) {
		self.gl.push_debug("Crosshair 2D");
		
		unsafe {
			self.gl.Enable(gl::BLEND);
			self.gl.BlendFunc(gl::ONE_MINUS_DST_COLOR, gl::ZERO);
		}
		
		let color = [1.0, 1.0, 1.0, 1.0];
		let scale = size / 2.0;
		
		let width = width as f32;
		let height = height as f32;
		
		let mut transform = cgmath::One::one();
		transform = transform * Matrix4::from_translation(cgmath::Vector3::<f32> {x: width/2.0, y: height/2.0, z: 0.0});
		transform = transform * Matrix4::from_nonuniform_scale(scale, scale, 0.0);
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
	
	pub fn draw(&self, camera: &cgmath::Matrix4<f32>, pos: &blocks::BlockCoord) {
		self.gl.push_debug("Crosshair 3D");
		
		unsafe {
			self.gl.Enable(gl::BLEND);
			self.gl.Disable(gl::CULL_FACE);
			self.gl.BlendFunc(gl::ONE_MINUS_DST_COLOR, gl::ZERO);
			self.gl.PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
		}
		
		let color = cgmath::Vector4::<f32> {x: 1.0, y: 1.0, z: 1.0, w: 1.0};
		
		let mut transform = cgmath::One::one();
		transform = transform * Matrix4::from_translation(HALF_VEC);
		transform = transform * Matrix4::from_translation(cgmath::vec3(pos.x as f32, pos.y as f32, pos.z as f32));
		transform = transform * Matrix4::from_scale(1.02);
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
