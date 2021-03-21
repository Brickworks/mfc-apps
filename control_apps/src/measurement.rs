use std::time::Instant;


pub struct Measurement<T> {
    pub value: T,
    pub timestamp: Instant,
}

impl<T> Measurement<T> {
    fn new(value: T, timestamp: Instant) -> Self {
        return Measurement{
            value,
            timestamp,
        }
    }
}
