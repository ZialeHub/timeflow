use std::ops::{Deref, DerefMut};

use chrono::{Datelike, Days, Duration, Local, Months, NaiveDateTime, Timelike};

pub const BASE_DATETIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

fn get_now_datetime() -> Result<NaiveDateTime, String> {
    let now = NaiveDateTime::parse_from_str(
        &Local::now().format(BASE_DATETIME_FORMAT).to_string(),
        BASE_DATETIME_FORMAT,
    )
    .map_err(|e| format!("Error while parsing now datetime: {e:?}"))?;
    Ok(now)
}

fn datetime_to_base_format(datetime: &str) -> Result<NaiveDateTime, String> {
    NaiveDateTime::parse_from_str(datetime, BASE_DATETIME_FORMAT).map_err(|e| e.to_string())
}

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
    pub fn new(datetime: impl ToString, format: impl ToString) -> Self {
        Self {
            datetime: NaiveDateTime::parse_from_str(&datetime.to_string(), &format.to_string())
                .unwrap(),
            format: format.to_string(),
        }
    }

    pub fn build(datetime: impl ToString) -> Self {
        Self {
            datetime: datetime_to_base_format(&datetime.to_string()).unwrap(),
            format: BASE_DATETIME_FORMAT.to_string(),
        }
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
            DateTimeUnit::Hour => Some(self.datetime + Duration::hours(value as i64)),
            DateTimeUnit::Minute => Some(self.datetime + Duration::minutes(value as i64)),
            DateTimeUnit::Second => Some(self.datetime + Duration::seconds(value as i64)),
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

    pub fn now() -> Self {
        Self {
            datetime: get_now_datetime().unwrap(),
            format: BASE_DATETIME_FORMAT.to_string(),
        }
    }

    pub fn is_in_future(&self) -> bool {
        self.datetime > get_now_datetime().unwrap()
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
                (self.datetime.timestamp() as i32 - lhs.datetime.timestamp() as i32) / 60 / 60 / 24
            }
            DateTimeUnit::Hour => {
                (self.datetime.timestamp() as i32 - lhs.datetime.timestamp() as i32) / 60 / 60
            }
            DateTimeUnit::Minute => {
                (self.datetime.timestamp() as i32 - lhs.datetime.timestamp() as i32) / 60
            }
            DateTimeUnit::Second => {
                self.datetime.timestamp() as i32 - lhs.datetime.timestamp() as i32
            }
        }
        .abs()
    }

    pub fn timestamp(&self) -> i64 {
        self.datetime.timestamp()
    }

    pub fn start_of_day(&self) -> Self {
        DateTime::from(
            self.datetime
                .with_hour(0)
                .unwrap()
                .with_minute(0)
                .unwrap()
                .with_second(0)
                .unwrap(),
        )
    }
}

impl From<NaiveDateTime> for DateTime {
    fn from(datetime: NaiveDateTime) -> Self {
        Self::build(datetime)
    }
}

impl From<u32> for DateTime {
    fn from(timestamp: u32) -> Self {
        Self::build(NaiveDateTime::from_timestamp_opt(timestamp as i64, 0).unwrap())
    }
}

impl From<u64> for DateTime {
    fn from(timestamp: u64) -> Self {
        Self::build(NaiveDateTime::from_timestamp_opt(timestamp as i64, 0).unwrap())
    }
}

impl From<(String, String)> for DateTime {
    fn from((datetime, format): (String, String)) -> Self {
        Self {
            datetime: NaiveDateTime::parse_from_str(&datetime, &format).unwrap(),
            format,
        }
    }
}

impl From<(&str, &str)> for DateTime {
    fn from((datetime, format): (&str, &str)) -> Self {
        Self {
            datetime: NaiveDateTime::parse_from_str(datetime, format).unwrap(),
            format: format.to_string(),
        }
    }
}

impl From<&str> for DateTime {
    fn from(datetime: &str) -> Self {
        Self::build(
            NaiveDateTime::parse_from_str(datetime, BASE_DATETIME_FORMAT)
                .expect("Error while parsing datetime"),
        )
    }
}

#[cfg(test)]
pub mod test {
    use super::*;

    #[test]
    fn test_add_overflow() {
        let mut datetime = DateTime::build(
            NaiveDateTime::parse_from_str("2023-10-09 00:00:00", BASE_DATETIME_FORMAT).unwrap(),
        );
        let new_datetime = datetime.update(DateTimeUnit::Day, i32::MAX);
        assert_eq!(
            new_datetime,
            Err("Cannot Add/Remove 2147483647 Day to/from 2023-10-09 00:00:00".to_string())
        );
    }

    #[test]
    fn test_add_one_year() {
        let mut datetime = DateTime::build(
            NaiveDateTime::parse_from_str("2023-10-09 00:00:00", BASE_DATETIME_FORMAT).unwrap(),
        );
        let new_datetime = datetime.update(DateTimeUnit::Year, 1);
        assert_eq!(new_datetime, Ok(()));
        assert_eq!(datetime.to_string(), "2024-10-09 00:00:00".to_string());
    }

    #[test]
    fn test_remove_one_year() {
        let mut datetime = DateTime::build(
            NaiveDateTime::parse_from_str("2023-10-09 00:00:00", BASE_DATETIME_FORMAT).unwrap(),
        );
        let new_datetime = datetime.update(DateTimeUnit::Year, -1);
        assert_eq!(new_datetime, Ok(()));
        assert_eq!(datetime.to_string(), "2022-10-09 00:00:00".to_string());
    }

    #[test]
    fn test_add_one_month() {
        let mut datetime = DateTime::build(
            NaiveDateTime::parse_from_str("2023-10-09 00:00:00", BASE_DATETIME_FORMAT).unwrap(),
        );
        let new_datetime = datetime.update(DateTimeUnit::Month, 1);
        assert_eq!(new_datetime, Ok(()));
        assert_eq!(datetime.to_string(), "2023-11-09 00:00:00".to_string());
    }

    #[test]
    fn test_remove_one_month() {
        let mut datetime = DateTime::build(
            NaiveDateTime::parse_from_str("2023-10-09 00:00:00", BASE_DATETIME_FORMAT).unwrap(),
        );
        let new_datetime = datetime.update(DateTimeUnit::Month, -1);
        assert_eq!(new_datetime, Ok(()));
        assert_eq!(datetime.to_string(), "2023-09-09 00:00:00".to_string());
    }

    #[test]
    fn test_add_one_day() {
        let mut datetime = DateTime::build(
            NaiveDateTime::parse_from_str("2023-10-09 00:00:00", BASE_DATETIME_FORMAT).unwrap(),
        );
        let new_datetime = datetime.update(DateTimeUnit::Day, 1);
        assert_eq!(new_datetime, Ok(()));
        assert_eq!(datetime.to_string(), "2023-10-10 00:00:00".to_string());
    }

    #[test]
    fn test_remove_one_day() {
        let mut datetime = DateTime::build(
            NaiveDateTime::parse_from_str("2023-10-09 00:00:00", BASE_DATETIME_FORMAT).unwrap(),
        );
        let new_datetime = datetime.update(DateTimeUnit::Day, -1);
        assert_eq!(new_datetime, Ok(()));
        assert_eq!(datetime.to_string(), "2023-10-08 00:00:00".to_string());
    }

    #[test]
    fn test_add_one_hour() {
        let mut datetime = DateTime::build(
            NaiveDateTime::parse_from_str("2023-10-09 00:00:00", BASE_DATETIME_FORMAT).unwrap(),
        );
        let new_datetime = datetime.update(DateTimeUnit::Hour, 1);
        assert_eq!(new_datetime, Ok(()));
        assert_eq!(datetime.to_string(), "2023-10-09 01:00:00".to_string());
    }

    #[test]
    fn test_remove_one_hour() {
        let mut datetime = DateTime::build(
            NaiveDateTime::parse_from_str("2023-10-09 00:00:00", BASE_DATETIME_FORMAT).unwrap(),
        );
        let new_datetime = datetime.update(DateTimeUnit::Hour, -1);
        assert_eq!(new_datetime, Ok(()));
        assert_eq!(datetime.to_string(), "2023-10-08 23:00:00".to_string());
    }

    #[test]
    fn test_add_one_minute() {
        let mut datetime = DateTime::build(
            NaiveDateTime::parse_from_str("2023-10-09 00:00:00", BASE_DATETIME_FORMAT).unwrap(),
        );
        let new_datetime = datetime.update(DateTimeUnit::Minute, 1);
        assert_eq!(new_datetime, Ok(()));
        assert_eq!(datetime.to_string(), "2023-10-09 00:01:00".to_string());
    }

    #[test]
    fn test_remove_one_minute() {
        let mut datetime = DateTime::build(
            NaiveDateTime::parse_from_str("2023-10-09 00:00:00", BASE_DATETIME_FORMAT).unwrap(),
        );
        let new_datetime = datetime.update(DateTimeUnit::Minute, -1);
        assert_eq!(new_datetime, Ok(()));
        assert_eq!(datetime.to_string(), "2023-10-08 23:59:00".to_string());
    }

    #[test]
    fn test_add_one_second() {
        let mut datetime = DateTime::build(
            NaiveDateTime::parse_from_str("2023-10-09 00:00:00", BASE_DATETIME_FORMAT).unwrap(),
        );
        let new_datetime = datetime.update(DateTimeUnit::Second, 1);
        assert_eq!(new_datetime, Ok(()));
        assert_eq!(datetime.to_string(), "2023-10-09 00:00:01".to_string());
    }

    #[test]
    fn test_remove_one_second() {
        let mut datetime = DateTime::build(
            NaiveDateTime::parse_from_str("2023-10-09 00:00:00", BASE_DATETIME_FORMAT).unwrap(),
        );
        let new_datetime = datetime.update(DateTimeUnit::Second, -1);
        assert_eq!(new_datetime, Ok(()));
        assert_eq!(datetime.to_string(), "2023-10-08 23:59:59".to_string());
    }
}
