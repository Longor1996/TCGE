use cgmath::Matrix4;
use cgmath::Vector3;

/// Represents a camera.
///
/// Due to coordinate-space differences, these functions should be used ***only*** for rendering.
pub trait Camera {
	/// Get the position of the camera.
	///
	/// This is the only function of this trait unaffected by coordinate-space shenanigans.
	fn get_gl_position(&self, interpolation: f32) -> Vector3<f32>;
	
	/// Get the rotation of the camera expressed as matrix.
	fn get_gl_rotation_matrix(&self, interpolation: f32) -> Matrix4<f32>;
	
	/// Given the size of a viewport and an interpolation factor, compute the Projection-Matrix for this camera.
	fn get_gl_projection_matrix(&self, viewport: (i32, i32), interpolation: f32) -> Matrix4<f32>;
	
	/// Given an interpolation factor, compute the View-Matrix for this camera.
	///
	/// If `translation` is `false`, the camera position is ignored in the computation.
	fn get_gl_view_matrix(&self, translation: bool, interpolation: f32) -> Matrix4<f32> {
		let rot: Matrix4<f32> = self.get_gl_rotation_matrix(interpolation).into();
		
		let mut out: Matrix4<f32> = rot;
		
		// This line further synchronizes the coordinate systems of OpenGL and basic
		// trigonometry, such that sin(theta) means the same in both systems.
		out = out * Matrix4::from_nonuniform_scale(1.0, 1.0, -1.0);
		
		if translation {
			let pos: Vector3<f32> = self.get_gl_position(interpolation);
			out = out * Matrix4::from_translation(-pos);
		}
		
		out
	}
}
