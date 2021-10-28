use chrono::{DateTime, Utc};

pub trait Termination {
    fn loop_started(&mut self);
    fn iteration_started(&mut self);
    fn should_terminate(&self) -> bool;
    fn iteration_ended(&mut self);
}

#[derive(Clone)]
pub struct TimeTermination {
    start_time: Option<DateTime<Utc>>,
    duration: chrono::Duration
}

impl TimeTermination {
    pub fn new(duration: chrono::Duration) -> TimeTermination {
        TimeTermination { start_time: None, duration }
    }
}

impl Termination for TimeTermination {
    fn loop_started(&mut self) {
        self.start_time = Some(Utc::now());
    }

    fn iteration_started(&mut self) { }

    fn should_terminate(&self) -> bool {
        self.start_time
            .map(|start_time| Utc::now() - start_time >= self.duration)
            .unwrap_or(true)
    }

    fn iteration_ended(&mut self) { }
}