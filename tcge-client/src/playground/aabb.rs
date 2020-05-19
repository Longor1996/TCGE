use std::ops::Neg;

#[derive(Copy, Clone, Debug)]
pub struct AxisAlignedBoundingBox {
	pub x_min: f32,
	pub x_max: f32,
	pub y_min: f32,
	pub y_max: f32,
	pub z_min: f32,
	pub z_max: f32,
}

impl AxisAlignedBoundingBox {
	
	pub fn from_min_max(min: nalgebra_glm::Vec3, max: nalgebra_glm::Vec3) -> Self {
		Self {
			x_min: min.x,
			y_min: min.y,
			z_min: min.z,
			x_max: max.x,
			y_max: max.y,
			z_max: max.z,
		}
	}
	
	pub fn from_extent(extent: f32) -> Self {
		Self {
			x_min: -extent,
			y_min: -extent,
			z_min: -extent,
			x_max: extent,
			y_max: extent,
			z_max: extent,
		}
	}
	
	pub fn from_position_radius_height(position: nalgebra_glm::Vec3, radius: f32, height: f32) -> Self {
		Self {
			x_min: &position.x - radius,
			y_min: &position.y - height,
			z_min: &position.z - radius,
			x_max: &position.x + radius,
			y_max: &position.y + height,
			z_max: &position.z + radius,
		}
	}
	
	pub fn from_position_size(position: nalgebra_glm::Vec3, size: nalgebra_glm::Vec3) -> Self {
		Self {
			x_min: position.x,
			y_min: position.y,
			z_min: position.z,
			x_max: &position.x + &size.x,
			y_max: &position.y + &size.y,
			z_max: &position.z + &size.z,
		}
	}
}

impl Default for AxisAlignedBoundingBox {
	fn default() -> Self {
		Self {
			x_min: -1.0,
			y_min: -1.0,
			z_min: -1.0,
			x_max: 1.0,
			y_max: 1.0,
			z_max: 1.0,
		}
	}
}

impl AxisAlignedBoundingBox {
	
	pub fn min_vec(&self) -> nalgebra_glm::Vec3 {
		nalgebra_glm::Vec3::new(self.x_min, self.y_min, self.z_min)
	}
	
	pub fn max_vec(&self) -> nalgebra_glm::Vec3 {
		nalgebra_glm::Vec3::new(self.x_max, self.y_max, self.z_max)
	}
	
	pub fn min(&self, idx: usize) -> f32 {
		match idx {
			0 => self.x_min,
			1 => self.y_min,
			2 => self.z_min,
			_ => panic!("Invalid axis id: {}", idx)
		}
	}
	
	pub fn max(&self, idx: usize) -> f32 {
		match idx {
			0 => self.x_max,
			1 => self.y_max,
			2 => self.z_max,
			_ => panic!("Invalid axis id: {}", idx)
		}
	}
	
	pub fn x(&self) -> f32 {
		(self.x_min + self.x_max) / 2.0
	}
	
	pub fn y(&self) -> f32 {
		(self.y_min + self.y_max) / 2.0
	}
	
	pub fn z(&self) -> f32 {
		(self.z_min + self.z_max) / 2.0
	}
	
	/// Returns the length on the X-axis (width) of the box.
	pub fn w(&self) -> f32 {
		self.x_max - self.x_min
	}
	
	/// Returns the length on the Y-axis (height) of the box.
	pub fn h(&self) -> f32 {
		self.y_max - self.y_min
	}
	
	/// Returns the length on the Z-axis (depth) of the box.
	pub fn d(&self) -> f32 {
		self.z_max - self.z_min
	}
	
	pub fn center(&self) -> nalgebra_glm::Vec3 {
		nalgebra_glm::Vec3::new(
			(self.x_min + self.x_max) / 2.0,
			(self.y_min + self.y_max) / 2.0,
			(self.z_min + self.z_max) / 2.0
		)
	}
	
	pub fn dimensions(&self) -> nalgebra_glm::Vec3 {
		nalgebra_glm::Vec3::new(
			self.x_max - self.x_min,
			self.y_max - self.y_min,
			self.z_max - self.z_min
		)
	}
	
	pub fn extent(&self) -> nalgebra_glm::Vec3 {
		self.dimensions() / 2.0
	}
	
	pub fn contains(&self, p: nalgebra_glm::Vec3) -> bool {
		   p.x >= self.x_min && p.x <= self.x_max
		&& p.y >= self.y_min && p.y <= self.y_max
		&& p.z >= self.z_min && p.z <= self.z_max
	}
	
	pub fn contains_with_delta(&self, p: nalgebra_glm::Vec3, d: f32) -> bool {
		   p.x >= self.x_min-d && p.x <= self.x_max+d
		&& p.y >= self.y_min-d && p.y <= self.y_max+d
		&& p.z >= self.z_min-d && p.z <= self.z_max+d
	}
	
	/// Return the closest corner as seen from the given point `p`.
	pub fn nearest_corner(&self, p: nalgebra_glm::Vec3) -> nalgebra_glm::Vec3 {
		fn nearest(v: f32, a: f32, b: f32) -> f32 {
			if (a - v).abs() < (b - v).abs() {a} else {b}
		};
		nalgebra_glm::Vec3::new(
			nearest(p.x, self.x_min, self.x_max),
			nearest(p.y, self.y_min, self.y_max),
			nearest(p.z, self.z_min, self.z_max)
		)
	}
	
	pub fn distance_squared(&self, other: &Self) -> f32 {
		let d = self.center() - other.center();
		d.x*d.x + d.y*d.y + d.z*d.z
	}
	
	pub fn distance(&self, other: &Self) -> f32 {
		let d = self.distance_squared(other);
		if d == 0.0 {0.0} else {d.sqrt()}
	}
	
	/// Returns a vector that tells how much `self` penetrates `other`.
	pub fn collision_vector(&self, other: &Self) -> nalgebra_glm::Vec3 {
		let (c1, c2) = (self.center(), other.center());
		nalgebra_glm::vec3(
			if c1.x < c2.x {self.x_max - other.x_min} else {self.x_min - other.x_max},
			if c1.y < c2.y {self.y_max - other.y_min} else {self.y_min - other.y_max},
			if c1.z < c2.z {self.z_max - other.z_min} else {self.z_min - other.z_max}
		)
	}
	
	/// Returns the minkowsky difference between two AABB's.
	pub fn minkowsky_diff(&self, other: &Self) -> Self {
		let sd = self.dimensions();
		let od = self.dimensions();
		Self {
			x_min: other.x_min - self.x_min - sd.x,
			y_min: other.y_min - self.y_min - sd.y,
			z_min: other.z_min - self.z_min - sd.z,
			x_max: sd.x + od.x,
			y_max: sd.y + od.y,
			z_max: sd.z + od.z,
		}
	}
	
	pub fn segment_intersection_indices(&self, start: nalgebra_glm::Vec3, end: nalgebra_glm::Vec3, ti1: Option<f32>, ti2: Option<f32>) -> Option<(f32, f32, nalgebra_glm::Vec3, nalgebra_glm::Vec3)> {
		
		let mut ti1 = ti1.unwrap_or(0.0);
		let mut ti2 = ti2.unwrap_or(1.0);
		
		let d: nalgebra_glm::Vec3 = end - start;
		
		let mut n1 = nalgebra_glm::Vec3::new(0.0, 0.0, 0.0);
		let mut n2 = nalgebra_glm::Vec3::new(0.0, 0.0, 0.0);
		
		for side in 0..6 {
			let (nx, ny, nz, p, q) = match side {
				0 => {
					(-1.0, 0.0, 0.0, -d.x, &start.x - self.x_min)
				},
				1 => {
					(1.0, 0.0, 0.0, d.x, self.x_max - &start.x)
				},
				2 => {
					(0.0, -1.0, 0.0, -d.y, &start.y - self.y_min)
				},
				3 => {
					(0.0, 1.0, 0.0, d.y, self.y_max - &start.y)
				},
				4 => {
					(0.0, 0.0, -1.0, -d.z, &start.z - self.z_min)
				},
				5 => {
					(0.0, 0.0, 1.0, d.z, self.z_max - &start.z)
				},
				_ => panic!()
			};
			
			if p == 0.0 {
				if q <= 0.0 {
					return None;
				}
			} else {
				let r = q / p;
				if p < 0.0 {
					// p < 0.0
					if r > ti2 {
						return None;
					} else if r > ti1 {
						ti1 = r;
						n1 = nalgebra_glm::Vec3::new(nx, ny, nz);
					}
				} else {
					// p > 0.0
					if r < ti1 {
						return None;
					} else if r < ti2 {
						ti2 = r;
						n2 = nalgebra_glm::Vec3::new(nx, ny, nz);
					}
				}
			}
		}
		
		Some((ti1, ti2, n1, n2))
	}
	
}

impl AxisAlignedBoundingBox {
	pub fn intersect(&self, other: &Self) -> bool {
		if self.x_max < other.x_min || self.x_min > other.x_max { return false; }
		if self.y_max < other.y_min || self.y_min > other.y_max { return false; }
		if self.z_max < other.z_min || self.z_min > other.z_max { return false; }
		true
	}
	
	pub fn intersection_x(&self, other: &Self, mut delta: f32) -> f32 {
		if other.y_max > self.y_min && other.y_min < self.y_max {
			if other.z_max > self.z_min && other.z_min < self.z_max {
				let mut d1 = 0.0;
				
				if delta > 0.0 && other.x_max <= self.x_min {
					d1 = self.x_min - other.x_max;
					if d1 < delta {
						delta = d1;
					}
				}
				
				if delta < 0.0 && other.x_min >= self.x_max {
					d1 = self.x_max - other.x_min;
					if d1 > delta {
						delta = d1;
					}
				}
				
				delta
			} else {
				delta
			}
		} else {
			delta
		}
	}
	
	pub fn intersection_y(&self, other: &Self, mut delta: f32) -> f32 {
		if other.x_max > self.x_min && other.x_min < self.x_max {
			if other.z_max > self.z_min && other.z_min < self.z_max {
				let mut d1 = 0.0;
				
				if delta > 0.0 && other.y_max <= self.y_min {
					d1 = self.y_min - other.y_max;
					if d1 < delta {
						delta = d1;
					}
				}
				
				if delta < 0.0 && other.y_min >= self.y_max {
					d1 = self.y_max - other.y_min;
					if d1 > delta {
						delta = d1;
					}
				}
				
				delta
			} else {
				delta
			}
		} else {
			delta
		}
	}
	
	pub fn intersection_z(&self, other: &Self, mut delta: f32) -> f32 {
		if other.x_max > self.x_min && other.x_min < self.x_max {
			if other.y_max > self.y_min && other.y_min < self.y_max {
				let mut d1 = 0.0;
				
				if delta > 0.0 && other.z_max <= self.z_min {
					d1 = self.z_min - other.z_max;
					if d1 < delta {
						delta = d1;
					}
				}
				
				if delta < 0.0 && other.z_min >= self.z_max {
					d1 = self.z_max - other.z_min;
					if d1 > delta {
						delta = d1;
					}
				}
				
				delta
			} else {
				delta
			}
		} else {
			delta
		}
	}
}

impl AxisAlignedBoundingBox {
	pub fn sweep_self(a: &Self, av: &nalgebra_glm::Vec3, b: &Self) -> Option<(f32, nalgebra_glm::Vec3, nalgebra_glm::Vec3, nalgebra_glm::Vec3)> {
		
		// Cant sweep if already intersecting
		if a.intersect(b) {
			return None;
		}
		
		if av.x == 0.0 && av.y == 0.0 && av.z == 0.0 {
			return None;
		}
		
		let v = av;
		
		// Treat b as stationary, so invert v to get relative velocity
		let v = v.neg();
		
		let mut hit_time = 0.0;
		let mut out_time = 1.0;
		
		let mut overlap_time = nalgebra_glm::Vec3::new(0.0, 0.0, 0.0);
		
		//=================================
		
		// X axis overlap
		if v.x < 0.0 {
			if b.x_max < a.x_min { return None; }
			if b.x_max > a.x_min { out_time = ((a.x_min - b.x_max) / v.x).min(out_time); }
			
			if a.x_max < b.x_min
			{
				overlap_time.x = (a.x_max - b.x_min) / v.x;
				hit_time = overlap_time.x.max(hit_time);
			}
		} else if v.x > 0.0
		{
			if b.x_min > a.x_max { return None; }
			if a.x_max > b.x_min { out_time = ((a.x_max - b.x_min) / v.x).min(out_time); }
			
			if b.x_max < a.x_min
			{
				overlap_time.x = (a.x_min - b.x_max) / v.x;
				hit_time = overlap_time.x.max(hit_time);
			}
		}
		
		if hit_time > out_time { return None; }
		
		//=================================
		
		// Y axis overlap
		if v.y < 0.0 {
			if b.y_max < a.y_min { return None; }
			if b.y_max > a.y_min { out_time = ((a.y_min - b.y_max) / v.y).min(out_time); }
			
			if a.y_max < b.y_min
			{
				overlap_time.y = (a.y_max - b.y_min) / v.y;
				hit_time = overlap_time.y.max(hit_time);
			}
		} else if v.y > 0.0
		{
			if b.y_min > a.y_max { return None; }
			if a.y_max > b.y_min { out_time = ((a.y_max - b.y_min) / v.y).min(out_time); }
			
			if b.y_max < a.y_min
			{
				overlap_time.y = (a.y_min - b.y_max) / v.y;
				hit_time = overlap_time.y.max(hit_time);
			}
		}
		
		if hit_time > out_time { return None; }
		
		//=================================
		
		// Z axis overlap
		if v.z < 0.0 {
			if b.z_max < a.z_min { return None; }
			if b.z_max > a.z_min { out_time = ((a.z_min - b.z_max) / v.z).min(out_time); }
			
			if a.z_max < b.z_min
			{
				overlap_time.z = (a.z_max - b.z_min) / v.z;
				hit_time = overlap_time.z.max(hit_time);
			}
		} else if v.z > 0.0
		{
			if b.z_min > a.z_max { return None; }
			if a.z_max > b.z_min { out_time = ((a.z_max - b.z_min) / v.z).min(out_time); }
			
			if b.z_max < a.z_min
			{
				overlap_time.z = (a.z_min - b.z_max) / v.z;
				hit_time = overlap_time.z.max(hit_time);
			}
		}
		
		if hit_time > out_time { return None; }
		
		//=================================
		
		// Scale resulting velocity by normalized hit time
		let out_velo = v.neg() * hit_time;
		
		let hit_norm = if overlap_time.x > overlap_time.y {
			// y is out
			if overlap_time.x > overlap_time.z {
				nalgebra_glm::Vec3::new(v.x.signum(), 0.0, 0.0)
			} else {
				nalgebra_glm::Vec3::new(0.0, 0.0, v.z.signum())
			}
		} else {
			// x is out
			if overlap_time.y > overlap_time.z {
				nalgebra_glm::Vec3::new(0.0, v.y.signum(), 0.0)
			} else {
				nalgebra_glm::Vec3::new(0.0, 0.0, v.z.signum())
			}
		};
		
		Some((hit_time, overlap_time, out_velo, hit_norm))
	}
}

#[derive(Debug, PartialEq)]
pub struct AxisAlignedBoundingBoxIntersection {
	pub overlaps: bool,
	pub ti: f32,
	pub mov: nalgebra_glm::Vec3,
	pub normal: Option<nalgebra_glm::Vec3>,
	pub touch: nalgebra_glm::Vec3,
	pub distance: f32,
}

impl std::cmp::PartialOrd for AxisAlignedBoundingBoxIntersection{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.ti == other.ti {
			return self.distance.partial_cmp(&other.distance);
		}
		return self.ti.partial_cmp(&other.ti)
    }
}

impl AxisAlignedBoundingBox {
	pub fn detect_collision(a: &Self, b: &Self, goal: nalgebra_glm::Vec3) -> Option<AxisAlignedBoundingBoxIntersection> {
		
		let d = goal - a.min_vec();
		
		let i = a.minkowsky_diff(b);
		
		let mut ti: Option<f32> = None;
		let mut n: Option<nalgebra_glm::Vec3> = None;
		
		let mut overlaps = false;
		
		const DELTA: f32 = 1e-10;
		
		if i.contains_with_delta(nalgebra_glm::Vec3::new(0.0, 0.0, 0.0), DELTA) {
			
			let p = i.nearest_corner(nalgebra_glm::Vec3::new(0.0, 0.0, 0.0));
			
			// Calculate the volume of the intersection...
			let adim = a.dimensions();
			let wi = adim.x.min(p.x.abs());
			let hi = adim.y.min(p.y.abs());
			let di = adim.z.min(p.z.abs());
			
			ti = Some(wi * hi * di * -1.0);
			overlaps = true;
		} else {
			//
			
			let si = i.segment_intersection_indices(
				nalgebra_glm::Vec3::new(0.0, 0.0, 0.0),
				d,
				Some(-std::f32::INFINITY),
				Some(std::f32::INFINITY)
			);
			
			if let Some((ti1, ti2, n1, _)) = si {
				
				if ti1 < 1.0
					&& ((ti1 - ti2).abs() >= DELTA) // prevents corner intersection
					&& (0.0 < ti1 + DELTA || (0.0 == ti1 && ti2 > 0.0))
				{
					ti = Some(ti1);
					n = Some(n1);
					overlaps = false;
				}
			}
		}
		
		let ti = match ti {
			Some(ti) => ti,
			None => return None
		};
		
		let mut t: nalgebra_glm::Vec3;
		
		if overlaps {
			if d.x == 0.0 && d.x == 0.0 && d.z == 0.0 {
				// intersecting and not moving - use minimum displacement vector
				
				let mut p = i.nearest_corner(nalgebra_glm::Vec3::new(0.0, 0.0, 0.0));
				
				if p.x.abs() <= p.y.abs() && p.x.abs() <= p.z.abs() {
					p.y = 0.0;
					p.z = 0.0;
				} else if p.y.abs() <= p.z.abs() {
					p.x = 0.0;
					p.z = 0.0;
				} else {
					p.x = 0.0;
					p.y = 0.0;
				}
				
				n = Some(nalgebra_glm::Vec3::new(p.x.signum(), p.y.signum(), p.z.signum()));
				t = nalgebra_glm::Vec3::new(
					a.x_min + p.x,
					a.y_min + p.y,
					a.z_min + p.z
				);
			} else {
				// intersecting and moving - move in the opposite direction
				
				let (ti1, _, n1, _) = match i.segment_intersection_indices(
					nalgebra_glm::Vec3::new(0.0, 0.0, 0.0),
					d,
					Some(-std::f32::INFINITY),
					Some(1.0)
				) {
					Some(si) => si,
					None => return None
				};
				
				n = Some(n1);
				t = nalgebra_glm::Vec3::new(
					a.x_min + d.x * ti1,
					a.y_min + d.y * ti1,
					a.z_min + d.z * ti1
				);
			}
		} else {
			// -- tunnel
			t = nalgebra_glm::Vec3::new(
				a.x_min + d.x * ti,
				a.y_min + d.y * ti,
				a.z_min + d.z * ti
			);
		}
		
		Some(AxisAlignedBoundingBoxIntersection {
			overlaps,
			ti,
			mov: d,
			normal: n,
			touch: t,
			distance: a.distance_squared(b)
		})
	}
	
}

/*
impl AxisAlignedBoundingBox {
	pub fn sweep_self2(a: &Self, b: &Self, av: &nalgebra_glm::Vec3) {
		
		// --- Calculate Inverse Entry/Exit
		
		let (x_inv_entry, x_inv_exit) = if av.x > 0.0 {
			(
				b.x() - (a.x() + a.w()),
				(b.x() + b.w()) - a.x()
			)
		} else {
			(
				(b.x() + b.w()) - a.x(),
				b.x() - (a.x() + a.w())
			)
		};
		
		let (y_inv_entry, y_inv_exit) = if av.y > 0.0 {
			(
				b.y() - (a.y() + a.h()),
				(b.y() + b.h()) - a.y()
			)
		} else {
			(
				(b.y() + b.h()) - a.y(),
				b.y() - (a.y() + a.h())
			)
		};
		
		let (z_inv_entry, z_inv_exit) = if av.z > 0.0 {
			(
				b.z() - (a.z() + a.d()),
				(b.z() + b.d()) - a.z()
			)
		} else {
			(
				(b.z() + b.d()) - a.z(),
				b.z() - (a.z() + a.d())
			)
		};
		
		// --- Calculate Entry/Exit by dividing the inverse by the velocity
		
		let (x_entry, x_exit) = if av.x == 0.0 {
			(f32::NEG_INFINITY, f32::INFINITY)
		} else {
			(x_inv_entry / av.x, x_inv_exit / av.x)
		};
		
		let (y_entry, y_exit) = if av.y == 0.0 {
			(f32::NEG_INFINITY, f32::INFINITY)
		} else {
			(y_inv_entry / av.y, y_inv_exit / av.y)
		};
		
		let (z_entry, z_exit) = if av.z == 0.0 {
			(f32::NEG_INFINITY, f32::INFINITY)
		} else {
			(z_inv_entry / av.z, z_inv_exit / av.z)
		};
		
		// --- Find earliest/latest times of collision.
		
		let entry_time = x_entry.max(y_entry).max(z_entry);
		let exit_time = x_entry.min(y_entry).min(z_entry);
		
		if entry_time > exit_time {
			//
		}
		
	}
}
*/