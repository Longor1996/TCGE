
pub struct GameloopState {
	//ticks_per_second: i32,
	skip_ticks: f64,
	max_frameskip: i32,
	next_game_tick: f64,
	loops: i32,
	interpolation: f64,
	frame_time: f64,
	frame_count: usize,
	tick_count: i32,
	last_chk: f64,
	last_fps: f64,
	last_tps: f64,
	print_timers: bool,
}

impl GameloopState {
	pub fn new(ticks_per_second: i32, print_timers: bool) -> GameloopState {
		GameloopState {
			skip_ticks: 1.0 / (ticks_per_second as f64),
			max_frameskip: 5,
			loops: 0,
			tick_count: 0,
			next_game_tick: 0.0,
			interpolation: 0.0,
			frame_time: 0.0,
			frame_count: 0,
			last_chk: 0.0,
			last_fps: 0.0,
			last_tps: 0.0,
			print_timers,
		}
	}
	
	pub fn next<GTC, GT, GD> (
		&mut self,
		gtc: GTC,
		mut game_tick: GT,
		mut game_draw: GD
	) where
		GTC: Fn() -> f64,
		GT: FnMut(f64),
		GD: FnMut(f64, f32),
	{
		let frame_start = gtc();
		self.loops = 0;
		
		while (gtc() > self.next_game_tick) && (self.loops < self.max_frameskip) {
			game_tick(gtc());
			
			self.next_game_tick += self.skip_ticks;
			self.loops += 1;
			self.tick_count += 1;
		}
		
		let now = gtc();
		let delta = now - self.next_game_tick;
		
		self.interpolation = (delta + self.skip_ticks) / self.skip_ticks;
		game_draw(now, self.interpolation as f32);
		
		let frame_end = gtc();
		self.frame_time = frame_end - frame_start;
		self.frame_count += 1;
		
		if frame_end - self.last_chk > 1.0 && self.frame_count > 10 {
			// calculate averages
			self.last_fps = self.frame_count as f64 / (frame_end - self.last_chk);
			self.last_tps = self.tick_count as f64 / (frame_end - self.last_chk);
			
			// reset timer
			self.frame_count = 0;
			self.tick_count = 0;
			self.last_chk = frame_end;
			
			if self.print_timers {
				debug!("{} FPS, {} TPS", self.last_fps, self.last_tps);
			}
		}
	}
	
	pub fn get_frame_time(&self) -> f64 {
		self.frame_time
	}
	
	pub fn get_frames_per_second(&self) -> f64 {
		self.last_fps
	}
	
	pub fn get_ticks_per_second(&self) -> f64 {
		self.last_tps
	}
}