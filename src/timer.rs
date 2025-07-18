use web_time::{Duration, Instant};

/// Manages all state related to frame timings.
pub struct FrameTimer {
    /// The accumulated frame count in the last full second.
    frame_count: u32,
    /// The smoothed out fps measure.
    fps: f32,
    /// How long the previous frame took to complete in seconds.
    delta_time: f32,

    /// The time of the last frame.
    last_frame: Instant,
    /// The time of the last full second.
    last_second: Instant,
}

impl FrameTimer {
    /// Creates a new [`FrameTimer`]
    pub fn new() -> Self {
        Self {
            frame_count: 0,
            fps: 0.0,
            delta_time: 0.0,
            last_frame: Instant::now(),
            last_second: Instant::now(),
        }
    }

    /// Updates the timer. Should be called once per frame.
    pub fn tick(&mut self) {
        self.frame_count += 1;

        let elapsed_frame_time = self.last_frame.elapsed();
        self.delta_time = elapsed_frame_time.as_secs_f32();
        self.last_frame = Instant::now();

        let elapsed_second_time = self.last_second.elapsed();
        if elapsed_second_time > Duration::from_secs(1) {
            self.fps = self.frame_count as f32 / elapsed_second_time.as_secs_f32();
            self.last_second = Instant::now();
            self.frame_count = 0;

            log::info!("running at {:.4} fps", self.fps);
        }
    }

    /// Returns the current delta time.
    pub fn dt(&self) -> f32 {
        self.delta_time
    }

    /// Returns the current smoothed fps.
    pub fn fps(&self) -> f32 {
        self.fps
    }
}

impl Default for FrameTimer {
    fn default() -> Self {
        Self::new()
    }
}
