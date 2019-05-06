
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

impl BlockRaycast {
	
	pub fn new_from_src_dir_len(src: cgmath::Vector3<f32>, dir: cgmath::Vector3<f32>, len: f32) -> BlockRaycast {
		let dst = src + (dir * len);
		BlockRaycast::new_from_src_dst(src, dst)
	}
	
	pub fn new_from_src_dst(src: cgmath::Vector3<f32>, dst: cgmath::Vector3<f32>) -> BlockRaycast {
		let gx0idx = src.x.floor();
		let gy0idx = src.y.floor();
		let gz0idx = src.z.floor();
		
		let gx1idx = dst.x.floor();
		let gy1idx = dst.y.floor();
		let gz1idx = dst.z.floor();
		
		let sx = BlockRaycast::psign(gx0idx, gx1idx);
		let sy = BlockRaycast::psign(gy0idx, gy1idx);
		let sz = BlockRaycast::psign(gz0idx, gz1idx);
		
		// Planes for each axis that we will next cross
		let gxp = gx0idx + (if gx1idx > gx0idx { 1.0 } else { 0.0 });
		let gyp = gy0idx + (if gy1idx > gy0idx { 1.0 } else { 0.0 });
		let gzp = gz0idx + (if gz1idx > gz0idx { 1.0 } else { 0.0 });
		
		// Only used for multiplying up the error margins
		let vx = if dst.x == src.x { 1.0 } else { dst.x - src.x};
		let vy = if dst.y == src.y { 1.0 } else { dst.y - src.y};
		let vz = if dst.z == src.z { 1.0 } else { dst.z - src.z};
		
		// Error is normalized to vx * vy * vz so we only have to multiply up
		let vxvy = vx * vy;
		let vxvz = vx * vz;
		let vyvz = vy * vz;
		
		// Error from the next plane accumulators, scaled up by vx*vy*vz
		//   gx0 + vx * rx === gxp
		//   vx * rx === gxp - gx0
		//   rx === (gxp - gx0) / vx
		let errx = (gxp - src.x) * vyvz;
		let erry = (gyp - src.y) * vxvz;
		let errz = (gzp - src.z) * vxvy;
		
		let derrx = sx * vyvz;
		let derry = sy * vxvz;
		let derrz = sz * vxvy;
		
		BlockRaycast {
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
	
	pub fn current(&self) -> (isize, isize, isize) {
		(
			self.gx as isize,
			self.gy as isize,
			self.gz as isize,
		)
	}
	
	pub fn previous(&self) -> (isize, isize, isize) {
		(
			self.lx as isize,
			self.ly as isize,
			self.lz as isize,
		)
	}
	
	pub fn step(&mut self) -> Option<(isize, isize, isize)> {
		if self.done {
			return None
		}
		
		let ret = (
			self.gx as isize,
			self.gy as isize,
			self.gz as isize,
		);
		
		if self.gx == self.gx1idx && self.gy == self.gy1idx && self.gz == self.gz1idx {
			self.done = true;
		}
		
		self.step_compute();
		self.visited += 1;
		return Some(ret)
	}
	
	fn step_compute(&mut self) {
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
