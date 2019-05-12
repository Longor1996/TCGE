use super::current_time_nanos;

pub struct TinyProfiler {
	t0: u128,
	t1: u128,
	tD: u128,
	tC: usize
}

impl TinyProfiler {
	pub fn new() -> TinyProfiler{
		TinyProfiler {
			t0: 0, t1: 0,
			tD: 0, tC: 0,
		}
	}
	
	pub fn start(&mut self) {
		self.t0 = current_time_nanos();
		self.t1 = self.t0;
	}
	
	pub fn end(&mut self) {
		self.t1 = current_time_nanos();
		self.tD += self.t1 - self.t0;
		self.tC += 1;
	}
	
	pub fn average(&self) -> u128 {
		if self.tC == 0 {
			return 0;
		}
		
		self.tD / self.tC as u128
	}
	
	pub fn total(&self) -> u128 {
		self.tD
	}
}
