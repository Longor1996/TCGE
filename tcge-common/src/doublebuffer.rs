pub struct DblBuf<T> {
	a: T,
	b: T,
	f: Flip,
}

impl<'d, T> DblBuf<T> {
	pub fn new(a: T, b: T) -> Self {
		Self {a, b, f: Flip::A}
	}
	
	pub fn swap(&mut self) {
		self.f = !self.f;
	}
	
	pub fn get_writer(&mut self) -> &'d mut T {
		match &self.f {
			Flip::A => unsafe {std::mem::transmute(&mut self.a)},
			Flip::B => unsafe {std::mem::transmute(&mut self.b)},
		}
	}
	
	pub fn get_reader(&self) -> &'d T {
		match &self.f {
			Flip::A => unsafe {std::mem::transmute(&self.b)},
			Flip::B => unsafe {std::mem::transmute(&self.a)},
		}
	}
}

#[derive(Clone, Copy)]
enum Flip {
	A, B
}

impl std::ops::Not for Flip {
	type Output = Flip;
	fn not(self) -> Self::Output {
		match self {
			Flip::A => Flip::B,
			Flip::B => Flip::A,
		}
	}
}
