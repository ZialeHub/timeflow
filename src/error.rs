#[derive(thiserror::Error, Debug, PartialEq)]
pub enum SpanError {
    #[error("Invalid Utc TryFrom / TryInto")]
    InvalidUtc,
    #[error("ParseFromStr: {0}")]
    ParseFromStr(#[from] chrono::ParseError),
    #[error("ParseFromTimestamp: {0}")]
    ParseFromTimestamp(String),
    #[error("ClearTime: {0}")]
    ClearTime(String),
    #[error("InvalidUpdate: {0}")]
    InvalidUpdate(String),
    #[error("Invalid datetime: {0}-{1}-{2} {3}:{4}:{5}")]
    InvalidDateTime(i32, u32, u32, u32, u32, u32),
    #[error("Invalid date: {0}-{1}-{2}")]
    InvalidDate(i32, u32, u32),
    #[error("Invalid time: {0}:{1}:{2}")]
    InvalidTime(u32, u32, u32),
    #[cfg(feature = "date")]
    #[error("{1} ➤  {0}")]
    Date(#[source] Box<SpanError>, DateError),
    #[cfg(feature = "datetime")]
    #[error("{1} ➤  {0}")]
    DateTime(#[source] Box<SpanError>, DateTimeError),
    #[cfg(feature = "time")]
    #[error("{1} ➤  {0}")]
    Time(#[source] Box<SpanError>, TimeError),
}

#[cfg(feature = "date")]
#[derive(thiserror::Error, Debug, PartialEq)]
#[error("Date")]
pub struct DateError;

#[cfg(feature = "datetime")]
#[derive(thiserror::Error, Debug, PartialEq)]
#[error("DateTime")]
pub struct DateTimeError;

#[cfg(feature = "time")]
#[derive(thiserror::Error, Debug, PartialEq)]
#[error("Time")]
pub struct TimeError;

pub trait ErrorContext<T, E> {
    fn err_ctx(self, context: E) -> Result<T, SpanError>;
}

#[cfg(feature = "date")]
impl<T> ErrorContext<T, DateError> for Result<T, SpanError> {
    fn err_ctx(self, context: DateError) -> Result<T, SpanError> {
        self.map_err(|e| SpanError::Date(Box::new(e), context))
    }
}

#[cfg(feature = "datetime")]
impl<T> ErrorContext<T, DateTimeError> for Result<T, SpanError> {
    fn err_ctx(self, context: DateTimeError) -> Result<T, SpanError> {
        self.map_err(|e| SpanError::DateTime(Box::new(e), context))
    }
}

#[cfg(feature = "time")]
impl<T> ErrorContext<T, TimeError> for Result<T, SpanError> {
    fn err_ctx(self, context: TimeError) -> Result<T, SpanError> {
        self.map_err(|e| SpanError::Time(Box::new(e), context))
    }
}
