#![allow(rustdoc::private_intra_doc_links)]

use std::{
    ops::Deref,
    sync::{Arc, RwLock},
};

use lazy_static::lazy_static;

pub mod builder;
pub mod date;
pub mod datetime;
pub mod error;
pub mod prelude;
pub mod time;

lazy_static! {
    static ref BASE_DATE_FORMAT: Arc<RwLock<&'static str>> = Arc::new(RwLock::new("%Y-%m-%d"));
    static ref BASE_TIME_FORMAT: Arc<RwLock<&'static str>> = Arc::new(RwLock::new("%H:%M:%S"));
    static ref BASE_DATETIME_FORMAT: Arc<RwLock<Option<&'static str>>> =
        Arc::new(RwLock::new(None));
}

impl BASE_DATE_FORMAT {
    pub fn get(&self) -> &'static str {
        &self.read().unwrap()
    }
}

impl BASE_TIME_FORMAT {
    pub fn get(&self) -> &'static str {
        &self.read().unwrap()
    }
}

impl BASE_DATETIME_FORMAT {
    pub fn get(&self) -> String {
        match self.read().unwrap().deref() {
            Some(format) => format.to_string(),
            None => format!("{} {}", BASE_DATE_FORMAT.get(), BASE_TIME_FORMAT.get()),
        }
    }
}

#[cfg(test)]
pub mod test {
    use super::*;
    use prelude::*;

    #[test]
    fn test_log_error_datetime_parse() {
        let datetime = DateTime::build("09-10-2023 00_00_00");
        let _ = datetime.inspect_err(|e| {
            assert_eq!(
                e.to_string(),
                "DateTime ➤  ParseFromStr: input contains invalid characters"
            )
        });
    }

    #[test]
    fn test_log_error_datetime_parse_timestamp() {
        let err = SpanError::DateTime(
            Box::new(SpanError::ParseFromTimestamp(
                "parse_from_timestamp error".to_string(),
            )),
            DateTimeError,
        );
        assert_eq!(
            err.to_string(),
            "DateTime ➤  ParseFromTimestamp: parse_from_timestamp error"
        )
    }

    #[test]
    fn test_log_error_datetime_invalid_update() {
        let err = SpanError::DateTime(
            Box::new(SpanError::InvalidUpdate(
                "Cannot add x Day to datetime error".to_string(),
            )),
            DateTimeError,
        );
        assert_eq!(
            err.to_string(),
            "DateTime ➤  InvalidUpdate: Cannot add x Day to datetime error"
        )
    }

    #[test]
    fn test_log_error_date_parse() {
        let datetime = Date::build("10-31-2023");
        let _ = datetime.inspect_err(|e| {
            assert_eq!(e.to_string(), "Date ➤  ParseFromStr: input is out of range")
        });
    }

    #[test]
    fn test_log_error_date_invalid_update() {
        let err = SpanError::Date(
            Box::new(SpanError::InvalidUpdate(
                "Cannot add x Month to date error".to_string(),
            )),
            DateError,
        );
        assert_eq!(
            err.to_string(),
            "Date ➤  InvalidUpdate: Cannot add x Month to date error"
        )
    }

    #[test]
    fn test_log_error_time_parse() {
        let time = Time::build("00_00_00");
        let _ = time.inspect_err(|e| {
            assert_eq!(
                e.to_string(),
                "Time ➤  ParseFromStr: input contains invalid characters"
            )
        });
    }

    #[test]
    fn test_log_error_time_invalid_update() {
        let err = SpanError::Time(
            Box::new(SpanError::InvalidUpdate(
                "Cannot add x Second to time error".to_string(),
            )),
            TimeError,
        );
        assert_eq!(
            err.to_string(),
            "Time ➤  InvalidUpdate: Cannot add x Second to time error"
        )
    }
    #[test]
    fn test_builder_format_default() -> Result<(), SpanError> {
        SpanBuilder::builder().build();
        let datetime = datetime::DateTime::build("2023-01-01 12:00:00")?;
        assert_eq!(datetime.to_string(), "2023-01-01 12:00:00");
        let date = date::Date::build("2023-01-01")?;
        assert_eq!(date.to_string(), "2023-01-01");
        let time = time::Time::build("12:00:00")?;
        assert_eq!(time.to_string(), "12:00:00");
        Ok(())
    }

    /// This test is ignored because it changes the global state of the date, time, and datetime
    /// Tests are running in parallel, and changing the global state might affect other tests
    #[test]
    #[ignore]
    fn test_builder_format_build() -> Result<(), SpanError> {
        SpanBuilder::builder()
            .datetime_format("%d/%m/%YT%H_%M_%S")
            .date_format("%d/%m/%Y")
            .time_format("%H_%M_%S")
            .build();
        datetime::DateTime::build("01/01/2023T12_00_00")?;
        date::Date::build("01/01/2023")?;
        time::Time::build("12_00_00")?;
        Ok(())
    }

    #[test]
    #[ignore]
    fn test_builder_format_build_ignored() -> Result<(), SpanError> {
        SpanBuilder::builder()
            .datetime_format("%d/%m/%YT%H_%M_%S")
            .date_format("%d/%m/%Y")
            .time_format("%H_%M_%S")
            .build();
        datetime::DateTime::new("2023-01-01 12:00:00", "%Y-%m-%d %H:%M:%S")?;
        date::Date::new("2023-01-01", "%Y-%m-%d")?;
        time::Time::new("12:00:00", "%H:%M:%S")?;
        Ok(())
    }

    #[test]
    #[ignore]
    fn test_builder_format_build_datetime_skipped() -> Result<(), SpanError> {
        SpanBuilder::builder()
            .date_format("%d/%m/%Y")
            .time_format("%H_%M_%S")
            .build();
        datetime::DateTime::build("01/01/2023 12_00_00")?;
        date::Date::build("01/01/2023")?;
        time::Time::build("12_00_00")?;
        Ok(())
    }
}
