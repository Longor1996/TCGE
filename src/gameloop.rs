
pub struct GameloopState {
    //ticks_per_second: i32,
    skip_ticks: f64,
    max_frameskip: i32,
    next_game_tick: f64,
    loops: i32,
    interpolation: f64,
    frame_time: f64,
}

impl GameloopState {
    pub fn new(ticks_per_second: i32) -> GameloopState {
        GameloopState {
            skip_ticks: 1.0 / (ticks_per_second as f64),
            max_frameskip: 5,
            loops: 0,
            next_game_tick: 0.0,
            interpolation: 0.0,
            frame_time: 0.0,
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
        }
        
        let now = gtc();
        let delta = now - self.next_game_tick;
    
        self.interpolation = (delta + self.skip_ticks) / self.skip_ticks;
        game_draw(now, self.interpolation as f32);
        
        let frame_end = gtc();
        self.frame_time = frame_end - frame_start;
    }
    
    pub fn get_frame_time(&self) -> f64 {
        self.frame_time
    }
}