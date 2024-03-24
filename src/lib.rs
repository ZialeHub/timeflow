#![allow(rustdoc::private_intra_doc_links)]

use std::{
    ops::Deref,
    sync::{Arc, RwLock},
};

use lazy_static::lazy_static;

pub mod date;
pub mod datetime;
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

/// Builder to set the default date, time, and datetime format
#[derive(Debug, Clone, Default)]
pub struct TimeBuilder {
    date_format: Option<&'static str>,
    time_format: Option<&'static str>,
    datetime_format: Option<&'static str>,
}

impl TimeBuilder {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_format_default() -> Result<(), String> {
        TimeBuilder::builder().build();
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
    fn test_builder_format_build() -> Result<(), String> {
        TimeBuilder::builder()
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
    fn test_builder_format_build_ignored() -> Result<(), String> {
        TimeBuilder::builder()
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
    fn test_builder_format_build_datetime_skipped() -> Result<(), String> {
        TimeBuilder::builder()
            .date_format("%d/%m/%Y")
            .time_format("%H_%M_%S")
            .build();
        datetime::DateTime::build("01/01/2023 12_00_00")?;
        date::Date::build("01/01/2023")?;
        time::Time::build("12_00_00")?;
        Ok(())
    }
}
