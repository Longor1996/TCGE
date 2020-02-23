use super::BlockDim;

/// Represents a ray travelling from a source to a destination trough a infinite uniform grid of unknown type.
pub struct BlockRaycast {
	gx: f32,
	gy: f32,
	gz: f32,
	lx: f32,
	ly: f32,
	lz: f32,
	gx1idx: f32,
	gy1idx: f32,
	gz1idx: f32,
	errx: f32,
	erry: f32,
	errz: f32,
	sx: f32,
	sy: f32,
	sz: f32,
	derrx: f32,
	derry: f32,
	derrz: f32,
	done: bool,
	visited: usize,
}

// Small helper to reduce clutter.
type VEC3 = (f32, f32, f32);

impl BlockRaycast {
	
	/// Create a new `BlockRaycast` given a source-position, direction and length/distance.
	pub fn new_from_src_dir_len(src: VEC3, dir: VEC3, len: f32) -> Self {
		Self::new_from_src_dst(src, (
			src.0 + (dir.0 * len),
			src.1 + (dir.1 * len),
			src.2 + (dir.2 * len),
		))
	}
	
	/// Create a new `BlockRaycast` given a source- and a destination-position.
	///
	/// This function prepares the necessary variables the algorithm requires to work.
	pub fn new_from_src_dst(src: VEC3, dst: VEC3) -> Self {
		#![allow(clippy::float_cmp)]
		
		let gx0idx = src.0.floor();
		let gy0idx = src.1.floor();
		let gz0idx = src.2.floor();
		
		let gx1idx = dst.0.floor();
		let gy1idx = dst.1.floor();
		let gz1idx = dst.2.floor();
		
		let sx = Self::psign(gx0idx, gx1idx);
		let sy = Self::psign(gy0idx, gy1idx);
		let sz = Self::psign(gz0idx, gz1idx);
		
		// Planes for each axis that we will next cross
		let gxp = gx0idx + (if gx1idx > gx0idx { 1.0 } else { 0.0 });
		let gyp = gy0idx + (if gy1idx > gy0idx { 1.0 } else { 0.0 });
		let gzp = gz0idx + (if gz1idx > gz0idx { 1.0 } else { 0.0 });
		
		// Only used for multiplying up the error margins
		let vx = if dst.0 == src.0 { 1.0 } else { dst.0 - src.0};
		let vy = if dst.1 == src.1 { 1.0 } else { dst.1 - src.1};
		let vz = if dst.2 == src.2 { 1.0 } else { dst.2 - src.2};
		
		// Error is normalized to vx * vy * vz so we only have to multiply up
		let vxvy = vx * vy;
		let vxvz = vx * vz;
		let vyvz = vy * vz;
		
		// Error from the next plane accumulators, scaled up by vx*vy*vz
		//   gx0 + vx * rx === gxp
		//   vx * rx === gxp - gx0
		//   rx === (gxp - gx0) / vx
		let errx = (gxp - src.0) * vyvz;
		let erry = (gyp - src.1) * vxvz;
		let errz = (gzp - src.2) * vxvy;
		
		let derrx = sx * vyvz;
		let derry = sy * vxvz;
		let derrz = sz * vxvy;
		
		Self {
			done: false,
			visited: 0,
			
			gx: gx0idx,
			gy: gy0idx,
			gz: gz0idx,
			lx: gx0idx,
			ly: gy0idx,
			lz: gz0idx,
			gx1idx, gy1idx, gz1idx,
			errx, erry, errz,
			sx, sy, sz,
			derrx, derry, derrz
		}
	}
	
	/// The current (voxel) position.
	pub fn current(&self) -> (BlockDim, BlockDim, BlockDim) {
		(
			self.gx as BlockDim,
			self.gy as BlockDim,
			self.gz as BlockDim,
		)
	}
	
	/// The previous (voxel) position.
	pub fn previous(&self) -> (BlockDim, BlockDim, BlockDim) {
		(
			self.lx as BlockDim,
			self.ly as BlockDim,
			self.lz as BlockDim,
		)
	}
	
	/// Wraps the state-handling of calculating the next step.
	pub fn step(&mut self) -> Option<(BlockDim, BlockDim, BlockDim)> {
		#![allow(clippy::float_cmp)]
		
		if self.done {
			return None
		}
		
		let ret = (
			self.gx as BlockDim,
			self.gy as BlockDim,
			self.gz as BlockDim,
		);
		
		if self.gx == self.gx1idx && self.gy == self.gy1idx && self.gz == self.gz1idx {
			self.done = true;
		}
		
		self.step_compute();
		self.visited += 1;
		Some(ret)
	}
	
	/// Calculates the next step using Bresenhams Line Algorithm (3D adaptation).
	fn step_compute(&mut self) {
		#![allow(clippy::float_cmp)]
		
		self.lx = self.gx;
		self.ly = self.gy;
		self.lz = self.gz;
		
		let xr = self.errx.abs();
		let yr = self.erry.abs();
		let zr = self.errz.abs();
		
		if (self.sx != 0.0) && (self.sy == 0.0 || xr < yr) && (self.sz == 0.0 || xr < zr) {
			self.gx += self.sx;
			self.errx += self.derrx;
		}
		else if (self.sy != 0.0) && (self.sz == 0.0 || yr < zr) {
			self.gy += self.sy;
			self.erry += self.derry;
		}
		else if self.sz != 0.0 {
			self.gz += self.sz;
			self.errz += self.derrz;
		}
	}
	
	/// Returns a float indicating which number is bigger.
	fn psign(a: f32, b: f32) -> f32 {
		if b > a {
			1.0
		} else if b < a {
			-1.0
		} else {
			0.0
		}
	}
	
}
