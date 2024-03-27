#[derive(thiserror::Error, Debug, PartialEq)]
pub enum SpanError {
    #[error("ParseFromStr: {0}")]
    ParseFromStr(#[from] chrono::ParseError),
    #[error("ParseFromTimestamp: {0}")]
    ParseFromTimestamp(String),
    #[error("ClearTime: {0}")]
    ClearTime(String),
    #[error("InvalidUpdate: {0}")]
    InvalidUpdate(String),
    #[error("{1} ➤  {0}")]
    Date(#[source] Box<SpanError>, DateError),
    #[error("{1} ➤  {0}")]
    DateTime(#[source] Box<SpanError>, DateTimeError),
    #[error("{1} ➤  {0}")]
    Time(#[source] Box<SpanError>, TimeError),
}

#[derive(thiserror::Error, Debug, PartialEq)]
#[error("Date")]
pub struct DateError;

#[derive(thiserror::Error, Debug, PartialEq)]
#[error("DateTime")]
pub struct DateTimeError;

#[derive(thiserror::Error, Debug, PartialEq)]
#[error("Time")]
pub struct TimeError;

pub trait ErrorContext<T, E> {
    fn err_ctx(self, context: E) -> Result<T, SpanError>;
}

impl<T> ErrorContext<T, DateError> for Result<T, SpanError> {
    fn err_ctx(self, context: DateError) -> Result<T, SpanError> {
        self.map_err(|e| SpanError::Date(Box::new(e), context))
    }
}

impl<T> ErrorContext<T, DateTimeError> for Result<T, SpanError> {
    fn err_ctx(self, context: DateTimeError) -> Result<T, SpanError> {
        self.map_err(|e| SpanError::DateTime(Box::new(e), context))
    }
}

impl<T> ErrorContext<T, TimeError> for Result<T, SpanError> {
    fn err_ctx(self, context: TimeError) -> Result<T, SpanError> {
        self.map_err(|e| SpanError::Time(Box::new(e), context))
    }
}
