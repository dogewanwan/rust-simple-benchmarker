use chrono::{DateTime, Utc};
use std::fmt::{Formatter, Display};

impl PreciseTime for DateTime<Utc> {
    fn now() -> Self {
        Utc::now()
    }

    fn is_before(&self, time: &DateTime<Utc>) -> bool {
        self <= &time
    }
}

pub trait PreciseTime {
    fn now() -> Self;
    fn is_before(&self, time: &DateTime<Utc>) -> bool;
}

pub struct SimpleLogger<Time: PreciseTime> {
    vectors: Vec<(Time, Option<Time>)>,
    current: Option<(Time, Option<Time>)>
}

impl Display for SimpleLogger<DateTime<Utc>> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let request_len = self.vectors
            .iter()
            .filter(|(_, x)| x.is_some())
            .count();
        let min_time = self.vectors
            .iter()
            .map(|(x, _)| x)
            .cloned()
            .min();
        let max_time = self.vectors
            .iter()
            .filter_map(|(_, y)| y.clone())
            .max();

        if let (Some(x), Some(y)) = (min_time, max_time) {
            let seconds = y - x;
            let rps = request_len as f64 / (seconds.num_seconds() as f64);

            write!(f, "RPS: {}", rps)
        } else {
            write!(f, "No results")
        }
    }
}

impl<T: PreciseTime> Default for SimpleLogger<T> {
    fn default() -> Self {
        SimpleLogger {
            vectors: vec![],
            current: None
        }
    }
}

impl<Time: PreciseTime, Error> ObservationLogger<Error> for SimpleLogger<Time> {
    fn log_start_of_request(&mut self) {
        self.current = Some((Time::now(), None));
    }

    fn log_end_of_request(&mut self, error: Option<Error>) {
        if let Some((start, _)) = self.current.take() {
            self.vectors.push((start, if error.is_none() { Some(Time::now()) } else { None }));
        }
    }

    fn merge(mut self, other: Self) -> Self {
        self.vectors.extend(other.vectors.into_iter());
        self
    }
}

pub trait ObservationLogger<Error> {
    fn log_start_of_request(&mut self);
    fn log_end_of_request(&mut self, error: Option<Error>);

    fn merge(self, other: Self) -> Self;
}