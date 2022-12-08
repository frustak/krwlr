use std::time::Instant;

pub struct Timer {
    instant: Instant,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            instant: Instant::now(),
        }
    }

    pub fn elapsed(&self) -> f64 {
        self.instant.elapsed().as_secs_f64()
    }
}
