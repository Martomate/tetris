use chrono::{DateTime, TimeDelta, Utc};

pub struct Clock {
    time: DateTime<Utc>,
}

impl Clock {
    pub fn now() -> Self {
        Self { time: Utc::now() }
    }

    pub fn update(&mut self, now: DateTime<Utc>) -> TimeDelta {
        let time_passed = now.signed_duration_since(self.time);
        self.time = now;
        time_passed
    }
}

pub struct Timer {
    time: TimeDelta,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            time: TimeDelta::zero(),
        }
    }

    pub fn advance(&mut self, time_passed: TimeDelta) {
        self.time += time_passed;
    }

    pub fn tick(&mut self, time_until_tick: TimeDelta) -> bool {
        let should_tick = self.time >= time_until_tick;
        if should_tick {
            self.time -= time_until_tick;
        }
        should_tick
    }

    pub fn reset(&mut self) {
        self.time = TimeDelta::zero();
    }
}
