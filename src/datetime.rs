use std::ops::{Deref, DerefMut};

use chrono::{Datelike, Days, Duration, Local, Months, NaiveDateTime, Timelike};

pub const BASE_DATETIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

#[derive(Debug, Clone)]
pub enum DateTimeUnit {
    Year,
    Month,
    Day,
    Hour,
    Minute,
    Second,
}

/// DateTime structure to handle datetime management
///
/// const BASE_DATETIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";
///
/// BASE_DATETIME_FORMAT is the default format for datetime
// TODO Find a way to implement Serialize and Deserialize for Time
#[derive(Debug, Clone)]
pub struct DateTime {
    datetime: NaiveDateTime,
    format: String,
}

impl std::fmt::Display for DateTime {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.datetime.format(&self.format))
    }
}

impl Deref for DateTime {
    type Target = NaiveDateTime;

    fn deref(&self) -> &Self::Target {
        &self.datetime
    }
}

impl DerefMut for DateTime {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.datetime
    }
}

impl DateTime {
    pub fn new(datetime: impl ToString, format: impl ToString) -> Result<Self, String> {
        let datetime =
            match NaiveDateTime::parse_from_str(&datetime.to_string(), &format.to_string()) {
                Ok(datetime) => datetime,
                Err(e) => return Err(format!("Error while parsing datetime: {e:?}")),
            };
        Ok(Self {
            datetime,
            format: format.to_string(),
        })
    }

    pub fn build(datetime: impl ToString) -> Result<Self, String> {
        let datetime = match NaiveDateTime::parse_from_str(&datetime.to_string(), BASE_DATETIME_FORMAT) {
            Ok(datetime) => datetime,
            Err(e) => return Err(format!("Error while parsing datetime into BASE_DATETIME_FORMAT '{BASE_DATETIME_FORMAT}': {e:?}")),
        };
        Ok(Self {
            datetime,
            format: BASE_DATETIME_FORMAT.to_string(),
        })
    }

    pub fn datetime(&self) -> NaiveDateTime {
        self.datetime
    }

    pub fn format(mut self, format: &str) -> Self {
        self.format = format.to_string();
        self
    }

    pub fn update(&mut self, unit: DateTimeUnit, value: i32) -> Result<(), String> {
        let datetime = match unit {
            DateTimeUnit::Year if value > 0 => self
                .datetime
                .checked_add_months(Months::new(value as u32 * 12)),
            DateTimeUnit::Year => self
                .datetime
                .checked_sub_months(Months::new(value.unsigned_abs() * 12)),
            DateTimeUnit::Month if value > 0 => {
                self.datetime.checked_add_months(Months::new(value as u32))
            }
            DateTimeUnit::Month => self
                .datetime
                .checked_sub_months(Months::new(value.unsigned_abs())),
            DateTimeUnit::Day if value > 0 => {
                self.datetime.checked_add_days(Days::new(value as u64))
            }
            DateTimeUnit::Day => self
                .datetime
                .checked_sub_days(Days::new(value.unsigned_abs() as u64)),
            DateTimeUnit::Hour => {
                Duration::try_hours(value as i64).map(|hours| self.datetime + hours)
            }
            DateTimeUnit::Minute => {
                Duration::try_minutes(value as i64).map(|minutes| self.datetime + minutes)
            }
            DateTimeUnit::Second => {
                Duration::try_seconds(value as i64).map(|seconds| self.datetime + seconds)
            }
        };
        match datetime {
            Some(datetime) => {
                self.datetime = datetime;
                Ok(())
            }
            None => Err(format!(
                "Cannot Add/Remove {} {:?} to/from {}",
                value, unit, self
            )),
        }
    }

    pub fn next(&mut self, unit: DateTimeUnit) -> Result<(), String> {
        self.update(unit, 1)
    }

    pub fn matches(&self, unit: DateTimeUnit, value: u32) -> bool {
        match unit {
            DateTimeUnit::Year => self.datetime.year() == value as i32,
            DateTimeUnit::Month => self.datetime.month() == value,
            DateTimeUnit::Day => self.datetime.day() == value,
            DateTimeUnit::Hour => self.datetime.hour() == value,
            DateTimeUnit::Minute => self.datetime.minute() == value,
            DateTimeUnit::Second => self.datetime.second() == value,
        }
    }

    pub fn now() -> Result<Self, String> {
        Self::build(Local::now().format(BASE_DATETIME_FORMAT))
    }

    pub fn is_in_future(&self) -> Result<bool, String> {
        let now = Self::build(Local::now().format(BASE_DATETIME_FORMAT))?;
        Ok(self.datetime > now.datetime)
    }

    pub fn elapsed(&self, lhs: &Self) -> Duration {
        self.datetime.signed_duration_since(lhs.datetime)
    }

    pub fn unit_in_between(&self, unit: DateTimeUnit, lhs: &Self) -> i32 {
        match unit {
            DateTimeUnit::Year => self.datetime.year() - lhs.datetime.year(),
            DateTimeUnit::Month => {
                self.datetime.year() * 12 + self.datetime.month() as i32
                    - (lhs.datetime.year() * 12 + lhs.datetime.month() as i32)
            }
            DateTimeUnit::Day => {
                (self.datetime.and_utc().timestamp() as i32
                    - lhs.datetime.and_utc().timestamp() as i32)
                    / 60
                    / 60
                    / 24
            }
            DateTimeUnit::Hour => {
                (self.datetime.and_utc().timestamp() as i32
                    - lhs.datetime.and_utc().timestamp() as i32)
                    / 60
                    / 60
            }
            DateTimeUnit::Minute => {
                (self.datetime.and_utc().timestamp() as i32
                    - lhs.datetime.and_utc().timestamp() as i32)
                    / 60
            }
            DateTimeUnit::Second => {
                self.datetime.and_utc().timestamp() as i32
                    - lhs.datetime.and_utc().timestamp() as i32
            }
        }
        .abs()
    }

    pub fn timestamp(&self) -> i64 {
        self.datetime.and_utc().timestamp()
    }

    pub fn clear_time(&self) -> Result<Self, String> {
        let datetime = self
            .datetime
            .with_hour(0)
            .inspect(|datetime| {
                datetime.with_minute(0);
            })
            .inspect(|datetime| {
                datetime.with_second(0);
            })
            .ok_or("Error while setting start of day".to_string())?;
        DateTime::try_from(datetime)
    }
}

impl TryFrom<NaiveDateTime> for DateTime {
    type Error = String;
    fn try_from(datetime: NaiveDateTime) -> Result<Self, Self::Error> {
        Self::build(datetime)
    }
}

impl TryFrom<i32> for DateTime {
    type Error = String;
    fn try_from(timestamp: i32) -> Result<Self, Self::Error> {
        let datetime = match chrono::DateTime::from_timestamp(timestamp as i64, 0) {
            Some(datetime) => datetime,
            None => return Err("Error while parsing timestamp from i32".to_string()),
        };
        Self::build(datetime)
    }
}

impl TryFrom<i64> for DateTime {
    type Error = String;
    fn try_from(timestamp: i64) -> Result<Self, Self::Error> {
        let datetime = match chrono::DateTime::from_timestamp(timestamp, 0) {
            Some(datetime) => datetime,
            None => return Err("Error while parsing timestamp from i64".to_string()),
        };
        Self::build(datetime)
    }
}

impl TryFrom<(String, String)> for DateTime {
    type Error = String;
    fn try_from((datetime, format): (String, String)) -> Result<Self, Self::Error> {
        Self::new(datetime, format)
    }
}

impl TryFrom<(&str, &str)> for DateTime {
    type Error = String;
    fn try_from((datetime, format): (&str, &str)) -> Result<Self, Self::Error> {
        Self::new(datetime, format)
    }
}

impl TryFrom<&str> for DateTime {
    type Error = String;
    fn try_from(datetime: &str) -> Result<Self, Self::Error> {
        Self::build(datetime)
    }
}

#[cfg(test)]
pub mod test {
    use super::*;

    #[test]
    fn test_datetime_add_overflow() -> Result<(), String> {
        let mut datetime = DateTime::build("2023-10-09 00:00:00")?;
        let new_datetime = datetime.update(DateTimeUnit::Day, i32::MAX);
        assert_eq!(
            new_datetime,
            Err("Cannot Add/Remove 2147483647 Day to/from 2023-10-09 00:00:00".to_string())
        );
        Ok(())
    }

    #[test]
    fn test_datetime_add_one_year() -> Result<(), String> {
        let mut datetime = DateTime::build("2023-10-09 00:00:00")?;
        let new_datetime = datetime.update(DateTimeUnit::Year, 1);
        assert_eq!(new_datetime, Ok(()));
        assert_eq!(datetime.to_string(), "2024-10-09 00:00:00".to_string());
        Ok(())
    }

    #[test]
    fn test_datetime_remove_one_year() -> Result<(), String> {
        let mut datetime = DateTime::build("2023-10-09 00:00:00")?;
        let new_datetime = datetime.update(DateTimeUnit::Year, -1);
        assert_eq!(new_datetime, Ok(()));
        assert_eq!(datetime.to_string(), "2022-10-09 00:00:00".to_string());
        Ok(())
    }

    #[test]
    fn test_datetime_add_one_month() -> Result<(), String> {
        let mut datetime = DateTime::build("2023-10-09 00:00:00")?;
        let new_datetime = datetime.update(DateTimeUnit::Month, 1);
        assert_eq!(new_datetime, Ok(()));
        assert_eq!(datetime.to_string(), "2023-11-09 00:00:00".to_string());
        Ok(())
    }

    #[test]
    fn test_datetime_remove_one_month() -> Result<(), String> {
        let mut datetime = DateTime::build("2023-10-09 00:00:00")?;
        let new_datetime = datetime.update(DateTimeUnit::Month, -1);
        assert_eq!(new_datetime, Ok(()));
        assert_eq!(datetime.to_string(), "2023-09-09 00:00:00".to_string());
        Ok(())
    }

    #[test]
    fn test_datetime_add_one_day() -> Result<(), String> {
        let mut datetime = DateTime::build("2023-10-09 00:00:00")?;
        let new_datetime = datetime.update(DateTimeUnit::Day, 1);
        assert_eq!(new_datetime, Ok(()));
        assert_eq!(datetime.to_string(), "2023-10-10 00:00:00".to_string());
        Ok(())
    }

    #[test]
    fn test_datetime_remove_one_day() -> Result<(), String> {
        let mut datetime = DateTime::build("2023-10-09 00:00:00")?;
        let new_datetime = datetime.update(DateTimeUnit::Day, -1);
        assert_eq!(new_datetime, Ok(()));
        assert_eq!(datetime.to_string(), "2023-10-08 00:00:00".to_string());
        Ok(())
    }

    #[test]
    fn test_datetime_add_one_hour() -> Result<(), String> {
        let mut datetime = DateTime::build("2023-10-09 00:00:00")?;
        let new_datetime = datetime.update(DateTimeUnit::Hour, 1);
        assert_eq!(new_datetime, Ok(()));
        assert_eq!(datetime.to_string(), "2023-10-09 01:00:00".to_string());
        Ok(())
    }

    #[test]
    fn test_datetime_remove_one_hour() -> Result<(), String> {
        let mut datetime = DateTime::build("2023-10-09 00:00:00")?;
        let new_datetime = datetime.update(DateTimeUnit::Hour, -1);
        assert_eq!(new_datetime, Ok(()));
        assert_eq!(datetime.to_string(), "2023-10-08 23:00:00".to_string());
        Ok(())
    }

    #[test]
    fn test_datetime_add_one_minute() -> Result<(), String> {
        let mut datetime = DateTime::build("2023-10-09 00:00:00")?;
        let new_datetime = datetime.update(DateTimeUnit::Minute, 1);
        assert_eq!(new_datetime, Ok(()));
        assert_eq!(datetime.to_string(), "2023-10-09 00:01:00".to_string());
        Ok(())
    }

    #[test]
    fn test_datetime_remove_one_minute() -> Result<(), String> {
        let mut datetime = DateTime::build("2023-10-09 00:00:00")?;
        let new_datetime = datetime.update(DateTimeUnit::Minute, -1);
        assert_eq!(new_datetime, Ok(()));
        assert_eq!(datetime.to_string(), "2023-10-08 23:59:00".to_string());
        Ok(())
    }

    #[test]
    fn test_datetime_add_one_second() -> Result<(), String> {
        let mut datetime = DateTime::build("2023-10-09 00:00:00")?;
        let new_datetime = datetime.update(DateTimeUnit::Second, 1);
        assert_eq!(new_datetime, Ok(()));
        assert_eq!(datetime.to_string(), "2023-10-09 00:00:01".to_string());
        Ok(())
    }

    #[test]
    fn test_datetime_remove_one_second() -> Result<(), String> {
        let mut datetime = DateTime::build("2023-10-09 00:00:00")?;
        let new_datetime = datetime.update(DateTimeUnit::Second, -1);
        assert_eq!(new_datetime, Ok(()));
        assert_eq!(datetime.to_string(), "2023-10-08 23:59:59".to_string());
        Ok(())
    }
}
