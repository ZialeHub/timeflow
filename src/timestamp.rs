use std::ops::Deref;

/// Timestamp in milliseconds
pub struct TimestampMilli(i64);
impl Deref for TimestampMilli {
    type Target = i64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<i64> for TimestampMilli {
    fn from(timestamp: i64) -> Self {
        Self(timestamp)
    }
}

/// Timestamp in microseconds (1 millisecond = 1_000 microseconds)
pub struct TimestampMicro(i64);
impl Deref for TimestampMicro {
    type Target = i64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<i64> for TimestampMicro {
    fn from(timestamp: i64) -> Self {
        Self(timestamp)
    }
}

/// Timestamp in nanoseconds (1 millisecond = 1_000_000 nanoseconds)
pub struct TimestampNano(i64);
impl Deref for TimestampNano {
    type Target = i64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<i64> for TimestampNano {
    fn from(timestamp: i64) -> Self {
        Self(timestamp)
    }
}
