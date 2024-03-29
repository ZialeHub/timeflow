use crate::{BASE_DATETIME_FORMAT, BASE_DATE_FORMAT, BASE_TIME_FORMAT};

/// Builder to set the default date, time, and datetime format
#[derive(Debug, Clone, Default)]
pub struct SpanBuilder {
    date_format: Option<&'static str>,
    time_format: Option<&'static str>,
    datetime_format: Option<&'static str>,
}

impl SpanBuilder {
    /// Create a new builder
    pub fn builder() -> Self {
        Self::default()
    }

    /// Setter for the date format
    pub fn date_format(&mut self, date_format: &'static str) -> &mut Self {
        self.date_format = Some(date_format);
        self
    }

    /// Setter for the time format
    pub fn time_format(&mut self, time_format: &'static str) -> &mut Self {
        self.time_format = Some(time_format);
        self
    }

    /// Setter for the datetime format
    pub fn datetime_format(&mut self, datetime_format: &'static str) -> &mut Self {
        self.datetime_format = Some(datetime_format);
        self
    }

    /// Consume the builder and set the default date, time, and datetime format
    pub fn build(&self) {
        match self.date_format {
            Some(date_format) => *BASE_DATE_FORMAT.write().unwrap() = date_format,
            None => *BASE_DATE_FORMAT.write().unwrap() = "%Y-%m-%d",
        }
        match self.time_format {
            Some(time_format) => *BASE_TIME_FORMAT.write().unwrap() = time_format,
            None => *BASE_TIME_FORMAT.write().unwrap() = "%H:%M:%S",
        }
        match self.datetime_format {
            Some(datetime_format) => *BASE_DATETIME_FORMAT.write().unwrap() = Some(datetime_format),
            None => *BASE_DATETIME_FORMAT.write().unwrap() = None,
        }
    }
}
