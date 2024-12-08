use chrono::Duration;

use crate::error::SpanError;

/// U is the Unit of the span
pub trait Span<U, F>
where
    Self: Sized,
{
    fn new(_: F, _: u32, _: u32) -> Result<Self, SpanError>;
    /// Setter for the format of the span
    fn format(self, format: impl ToString) -> Self;
    /// Update the value of the span by a given Unit (U) and value.
    fn update(&self, unit: U, value: i32) -> Result<Self, SpanError>;
    /// Get the value of the span incremented by a given Unit (U).
    fn next(&self, unit: U) -> Result<Self, SpanError>;
    /// Return if the span matches the given unit and value.
    fn matches(&self, unit: U, value: u32) -> bool;
    /// Return the value of the span at system time.
    fn now() -> Result<Self, SpanError>;
    fn is_in_future(&self) -> Result<bool, SpanError>;
    fn elapsed(&self, lhs: &Self) -> Duration;
    fn unit_elapsed(&self, rhs: &Self, unit: U) -> Result<i64, SpanError>;
    fn clear_unit(&self, unit: U) -> Result<Self, SpanError>;
}
