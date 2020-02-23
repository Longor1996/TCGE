#[repr(u8)]
#[derive(Debug, Hash, Eq, PartialEq, Copy, Clone)]
pub enum Face {
	// None = 0,
	PositiveX = 1, NegativeX = 2,
	PositiveY = 3, NegativeY = 4,
	PositiveZ = 5, NegativeZ = 6,
	EveryDir  = 7
}

impl Face {
	pub fn id(&self) -> u8 {
		unsafe { ::std::mem::transmute(*self) }
	}
	
	pub fn uid(&self) -> usize {
		self.id() as usize
	}
	
	pub fn normal(&self) -> (f32, f32, f32) {
		match self {
			Face::PositiveY => ( 0.0, 1.0, 0.0),
			Face::NegativeY => ( 0.0,-1.0, 0.0),
			Face::PositiveX => ( 1.0, 0.0, 0.0),
			Face::NegativeX => (-1.0, 0.0, 0.0),
			Face::PositiveZ => ( 0.0, 0.0, 1.0),
			Face::NegativeZ => ( 0.0, 0.0,-1.0),
			Face::EveryDir => (0.0, 0.0, 0.0),
		}
	}
}
