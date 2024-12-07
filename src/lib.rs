#![allow(rustdoc::private_intra_doc_links)]
#[cfg(not(any(feature = "time", feature = "date", feature = "datetime")))]
compile_error!("At least one feature must be enabled: 'time', 'date', or 'datetime'");

pub mod builder;
#[cfg(feature = "date")]
pub mod date_module;
#[cfg(feature = "datetime")]
pub mod datetime_module;
pub mod error;
pub mod prelude;
#[cfg(feature = "time")]
pub mod time_module;

#[cfg(feature = "datetime")]
use std::ops::Deref;
use std::sync::{LazyLock, RwLock};

#[cfg(feature = "date")]
pub use date_module::date;
#[cfg(feature = "datetime")]
pub use datetime_module::datetime;
#[cfg(feature = "time")]
pub use time_module::time;

pub trait GetInner<T: std::fmt::Display> {
    fn get(&self) -> T;
}

pub(crate) type BaseFormat<T> = LazyLock<RwLock<T>>;

#[cfg(any(feature = "date", feature = "time"))]
impl GetInner<&'static str> for BaseFormat<&'static str> {
    fn get(&self) -> &'static str {
        &self.read().unwrap()
    }
}

#[cfg(feature = "datetime")]
impl GetInner<String> for BaseFormat<Option<&'static str>> {
    fn get(&self) -> String {
        match self.read().unwrap().deref() {
            Some(format) => format.to_string(),
            #[cfg(all(feature = "date", feature = "time"))]
            None => format!(
                "{} {}",
                crate::date::BASE_DATE_FORMAT.get(),
                crate::time::BASE_TIME_FORMAT.get()
            ),
            #[cfg(all(not(feature = "date"), feature = "time"))]
            None => format!("%Y-%m-%d {}", BASE_TIME_FORMAT.get()),
            #[cfg(all(feature = "date", not(feature = "time")))]
            None => format!("{} %H:%M:%S", BASE_DATE_FORMAT.get()),
            #[cfg(not(all(feature = "date", feature = "time")))]
            None => "%Y-%m-%d %H:%M:%S".to_string(),
        }
    }
}

#[cfg(test)]
pub mod test {
    use super::*;
    use prelude::*;

    #[test]
    fn log_error_datetime_parse() {
        let datetime = DateTime::build("09-10-2023 00_00_00");
        let _ = datetime.inspect_err(|e| {
            assert_eq!(
                e.to_string(),
                "DateTime ➤  ParseFromStr: input contains invalid characters"
            )
        });
    }

    #[test]
    fn log_error_datetime_parse_timestamp() {
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
    fn log_error_datetime_invalid_update() {
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
    fn log_error_date_parse() {
        let datetime = Date::build("10-31-2023");
        let _ = datetime.inspect_err(|e| {
            assert_eq!(e.to_string(), "Date ➤  ParseFromStr: input is out of range")
        });
    }

    #[test]
    fn log_error_date_invalid_update() {
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
    fn log_error_time_parse() {
        let time = Time::build("00_00_00");
        let _ = time.inspect_err(|e| {
            assert_eq!(
                e.to_string(),
                "Time ➤  ParseFromStr: input contains invalid characters"
            )
        });
    }

    #[test]
    fn log_error_time_invalid_update() {
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
    fn builder_format_default() -> Result<(), SpanError> {
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
    fn builder_format_build() -> Result<(), SpanError> {
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
    fn builder_format_build_ignored() -> Result<(), SpanError> {
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
    fn builder_format_build_datetime_skipped() -> Result<(), SpanError> {
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
