use super::doublebuffer::DblBuf;
use std::cell::{UnsafeCell};
use std::time::Instant;

////////////////////////////////////////////////////////////////////////////////

pub fn profiler() -> &'static mut Profiler {
	PROF.with(|p|
		// This is VERY unsafe.
		unsafe { std::mem::transmute(p.get())}
	)
}

pub fn start_frame() {
	profiler().start_frame()
}

pub fn scope<F, V>(name: &'static str, func: F) -> V
	where F: FnOnce() -> V {
	let current = profiler().get_current();
	current.enter_noguard(name);
	let v = func();
	current.leave();
	v
}

pub fn end_frame() {
	profiler().end_frame()
}

////////////////////////////////////////////////////////////////////////////////

thread_local! {
	pub static PROF: UnsafeCell<Profiler> = UnsafeCell::new(Profiler::new());
}

pub struct Profiler {
	dblbuf: DblBuf<ProfilerTree>
}

impl Profiler {
	pub fn new() -> Self {
		let a = ProfilerTree::new();
		let b = ProfilerTree::new();
		Self {
			dblbuf: DblBuf::new(a, b)
		}
	}
	
	pub fn get_current(&mut self) -> &mut ProfilerTree {
		self.dblbuf.get_writer()
	}
	
	pub fn get_passive(&self) -> &ProfilerTree {
		self.dblbuf.get_reader()
	}
	
	pub fn start_frame(&mut self) {
		// Clear out old data!
		self.get_current().reset();
	}
	
	pub fn end_frame(&mut self) {
		self.get_current().leave();
		self.dblbuf.swap();
	}
}

pub struct ProfilerTree {
	pub nodes: Vec<ProfilerNode>,
	pub stack: Vec<usize>,
}

impl ProfilerTree {
	pub fn new() -> Self {
		Self {
			nodes: vec![ ProfilerNode::new(None, "root") ],
			stack: vec![ 0 ],
		}
	}
	
	pub fn reset(&mut self) {
		for node in self.nodes.iter_mut() {
			node.reset();
		}
		
		self.stack.push(0);
		self.nodes[0].enter();
	}
	
	pub fn enter(&mut self, name: &'static str) -> ProfilerGuard {
		self.enter_noguard(name);
		ProfilerGuard(self, true)
	}
	
	pub fn enter_noguard(&mut self, name: &'static str) {
		if self.nodes.len() > 1000 {
			panic!("This aint' supposed to happen...");
		}
		
		let mut curr = *self.stack.last().unwrap();
		
		if self.nodes[curr].name != name {
			let mut next: Option<usize> = None;
			for child in self.nodes[curr].childs.iter() {
				if self.nodes[*child].name == name {
					next = Some(*child);
				}
			}
			
			if let Some(next) = next {
				// Reuse the node...
				self.stack.push(next);
				curr = next;
			}
			else {
				// Create a new node...
				let next = self.nodes.len();
				self.nodes[curr].childs.push(next);
				self.nodes.push(ProfilerNode::new(Some(curr), name));
				self.stack.push(next);
				curr = next;
			}
		}
		
		self.nodes[curr].enter();
	}
	
	pub fn leave(&mut self) {
		let curr = *self.stack.last().unwrap();
		let curr = &mut self.nodes[curr];
		
		curr.leave();
		
		if let Some(_) = curr.parent {
			self.stack.pop();
		}
	}
	
	pub fn print(&self) {
		println!("Timing information for {}:", self.nodes[0].name);
		for child in &self.nodes[0].childs {
			if self.nodes[*child].calls > 0 {
				self.print_do(*child, 1);
			}
		}
	}
	
	pub fn print_do(&self, node: usize, depth: u32) {
		for _ in 0..depth {
			print!("\t");
		}
		
		let ns = Nanosec {inner: self.nodes[node].total_time};
		
		println!("{}: {}", self.nodes[node].name, ns);
		for child in &self.nodes[node].childs {
			self.print_do(*child, depth + 1);
		}
	}
}

pub struct ProfilerNode {
	// Label
	pub name: &'static str,
	
	// Structure
	pub parent: Option<usize>,
	pub childs: Vec<usize>,
	
	// Stats
	pub calls: u32,
	pub start_time: std::time::Instant,
	pub total_time: u128,
}

impl ProfilerNode {
	pub fn new(parent: Option<usize>, name: &'static str) -> Self {
		Self {
			name,
			parent,
			childs: vec![],
			calls: 0,
			start_time: Instant::now(),
			total_time: 0,
		}
	}
	
	pub fn reset(&mut self) {
		self.calls = 0;
		self.start_time = Instant::now();
		self.total_time = 0;
	}
	
	pub fn enter(&mut self) {
		self.calls += 1;
		self.start_time = Instant::now();
	}
	
	pub fn leave(&mut self) {
		self.total_time += Instant::now()
			.duration_since(self.start_time)
			.as_nanos();
	}
	
	pub fn get_time_as_nanosec(&self) -> Nanosec {
		Nanosec { inner: self.total_time }
	}
}

pub struct ProfilerGuard<'a>(&'a mut ProfilerTree, bool);
impl <'a> ProfilerGuard<'a> {
	pub fn force_leave(&mut self) {
		if self.1 {
			self.0.leave();
			self.1 = false;
		}
	}
}
impl <'a> Drop for ProfilerGuard<'a> {
	fn drop(&mut self) {
		if self.1 {
			self.0.leave();
			self.1 = false;
		}
	}
}

pub struct Nanosec {
	inner: u128
}

impl std::fmt::Display for Nanosec {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
		if self.inner < 1_000 {
			write!(f, "{}ns", self.inner)
		} else if self.inner < 1_000_000 {
			write!(f, "{:.1}us", self.inner as f64 / 1_000.)
		} else if self.inner < 1_000_000_000 {
			write!(f, "{:.1}ms", self.inner as f64 / 1_000_000.)
		} else {
			write!(f, "{:.1}s", self.inner as f64 / 1_000_000_000.)
		}
	}
}