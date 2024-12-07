#[cfg(feature = "date")]
use crate::date::BASE_DATE_FORMAT;
#[cfg(feature = "datetime")]
use crate::datetime::BASE_DATETIME_FORMAT;
#[cfg(feature = "time")]
use crate::time::BASE_TIME_FORMAT;

/// Builder to set the default date, time, and datetime format
///
/// # Attributes
/// - date_format: Option<&'static str>
/// - time_format: Option<&'static str>
/// - datetime_format: Option<&'static str>
#[derive(Debug, Clone, Default)]
pub struct SpanBuilder {
    #[cfg(feature = "date")]
    date_format: Option<&'static str>,
    #[cfg(feature = "time")]
    time_format: Option<&'static str>,
    #[cfg(feature = "datetime")]
    datetime_format: Option<&'static str>,
}

impl SpanBuilder {
    /// Create a SpanBuilder to personnalie the default date, time, and datetime format
    ///
    /// # Example
    /// ```rust,ignore
    /// let builder = SpanBuilder::builder()
    ///    .date_format("%d %m %Y")
    ///    .time_format("T%H:%M:%SZ.000")
    ///    .datetime_format("%Y-%m-%d %H:%M:%S")
    ///    .build();
    /// ```
    pub fn builder() -> Self {
        Self::default()
    }

    /// Setter for the date format
    #[cfg(feature = "date")]
    pub fn date_format(&mut self, date_format: &'static str) -> &mut Self {
        self.date_format = Some(date_format);
        self
    }

    /// Setter for the time format
    #[cfg(feature = "time")]
    pub fn time_format(&mut self, time_format: &'static str) -> &mut Self {
        self.time_format = Some(time_format);
        self
    }

    /// Setter for the datetime format
    #[cfg(feature = "datetime")]
    pub fn datetime_format(&mut self, datetime_format: &'static str) -> &mut Self {
        self.datetime_format = Some(datetime_format);
        self
    }

    /// Consume the builder and set the default date, time, and datetime format
    pub fn build(&self) {
        #[cfg(feature = "date")]
        match self.date_format {
            Some(date_format) => *BASE_DATE_FORMAT.write().unwrap() = date_format,
            None => *BASE_DATE_FORMAT.write().unwrap() = "%Y-%m-%d",
        }

        #[cfg(feature = "time")]
        match self.time_format {
            Some(time_format) => *BASE_TIME_FORMAT.write().unwrap() = time_format,
            None => *BASE_TIME_FORMAT.write().unwrap() = "%H:%M:%S",
        }

        #[cfg(feature = "datetime")]
        match self.datetime_format {
            Some(datetime_format) => *BASE_DATETIME_FORMAT.write().unwrap() = Some(datetime_format),
            None => *BASE_DATETIME_FORMAT.write().unwrap() = None,
        }
    }
}
