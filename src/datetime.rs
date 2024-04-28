use std::ops::{Deref, DerefMut};

use chrono::{Datelike, Days, Duration, Local, Months, NaiveDateTime, Timelike, Utc};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

use crate::{
    error::{DateTimeError, ErrorContext, SpanError},
    BASE_DATETIME_FORMAT,
};

/// [Serialize] the [NaiveDateTime] variable from [DateTime]
pub fn datetime_to_str<S: Serializer>(
    datetime: &NaiveDateTime,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    datetime
        .format(&BASE_DATETIME_FORMAT.get())
        .to_string()
        .serialize(serializer)
}

/// [Deserialize] the [NaiveDateTime] variable from [DateTime]
pub fn datetime_from_str<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
where
    D: Deserializer<'de>,
{
    let date: String = Deserialize::deserialize(deserializer)?;
    NaiveDateTime::parse_from_str(&date, &BASE_DATETIME_FORMAT.get()).map_err(de::Error::custom)
}

/// Unit to update [DateTime]
#[derive(Debug, Clone)]
pub enum DateTimeUnit {
    Year,
    Month,
    Day,
    Hour,
    Minute,
    Second,
}

/// Structure to handle datetime management
///
/// Use [BASE_DATETIME_FORMAT](static@BASE_DATETIME_FORMAT) as default format for datetime
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateTime {
    #[serde(
        serialize_with = "datetime_to_str",
        deserialize_with = "datetime_from_str"
    )]
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
    /// Create a new variable [DateTime] from the parameters `datetime` and `format`
    ///
    ///  See the [chrono::format::strftime] for the supported escape sequences of `format`.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let datetime = DateTime::new("05/17/2024T09_27_00", "%m/%d/%YT%H_%M_%S")?;
    /// ```
    ///
    /// # Errors
    ///
    /// Return an Err(_) if `datetime` is not formated with `format`
    pub fn new(datetime: impl ToString, format: impl ToString) -> Result<Self, SpanError> {
        let datetime =
            match NaiveDateTime::parse_from_str(&datetime.to_string(), &format.to_string()) {
                Ok(datetime) => datetime,
                Err(e) => return Err(SpanError::ParseFromStr(e)).err_ctx(DateTimeError),
            };
        Ok(Self {
            datetime,
            format: format.to_string(),
        })
    }

    /// Create a new variable [DateTime] from the parameter `datetime` formated with [BASE_DATETIME_FORMAT](static@BASE_DATETIME_FORMAT)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let datetime = DateTime::build("2023-05-17 09:05:12")?;
    /// ```
    ///
    /// # Errors
    ///
    /// Return an Err(_) if the given `datetime` is not formated with [BASE_DATETIME_FORMAT](static@BASE_DATETIME_FORMAT)
    pub fn build(datetime: impl ToString) -> Result<Self, SpanError> {
        let datetime =
            match NaiveDateTime::parse_from_str(&datetime.to_string(), &BASE_DATETIME_FORMAT.get())
            {
                Ok(datetime) => datetime,
                Err(e) => return Err(SpanError::ParseFromStr(e)).err_ctx(DateTimeError),
            };
        Ok(Self {
            datetime,
            format: BASE_DATETIME_FORMAT.get().to_string(),
        })
    }

    /// Getter for the datetime
    pub fn datetime(&self) -> NaiveDateTime {
        self.datetime
    }

    /// Setter for the format
    pub fn format(mut self, format: &str) -> Self {
        self.format = format.to_string();
        self
    }

    /// Function to increase / decrease the datetime [DateTime] with [DateTimeUnit]
    pub fn update(&mut self, unit: DateTimeUnit, value: i32) -> Result<(), SpanError> {
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
            None => Err(SpanError::InvalidUpdate(format!(
                "Cannot Add/Remove {} {:?} to/from {}",
                value, unit, self
            )))
            .err_ctx(DateTimeError),
        }
    }

    /// Go to the next [DateTimeUnit] from [DateTime]
    pub fn next(&mut self, unit: DateTimeUnit) -> Result<(), SpanError> {
        self.update(unit, 1)
    }

    /// Compare the [DateTimeUnit] from [DateTime] and value ([u32])
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

    /// Return the current [DateTime] from the system
    pub fn now() -> Result<Self, SpanError> {
        Self::build(Local::now().format(&BASE_DATETIME_FORMAT.get()))
    }

    /// Return a [bool] to know if the [DateTime] is in the future
    pub fn is_in_future(&self) -> Result<bool, SpanError> {
        let now = Self::build(Local::now().format(&BASE_DATETIME_FORMAT.get()))?;
        Ok(self.datetime > now.datetime)
    }

    /// Elapsed [Duration] between two [DateTime]
    pub fn elapsed(&self, lhs: &Self) -> Duration {
        self.datetime.signed_duration_since(lhs.datetime)
    }

    /// Number of [DateTimeUnit] between two [DateTime]
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

    /// Return the timestamp from the [DateTime]
    pub fn timestamp(&self) -> i64 {
        self.datetime.and_utc().timestamp()
    }

    /// Clear the time from the [DateTime]
    pub fn clear_time(&self) -> Result<Self, SpanError> {
        let datetime = self
            .datetime
            .with_hour(0)
            .and_then(|datetime| datetime.with_minute(0))
            .and_then(|datetime| datetime.with_second(0))
            .ok_or(SpanError::ClearTime(
                "Error while setting start of day".to_string(),
            ))
            .err_ctx(DateTimeError)?;
        DateTime::try_from(datetime)
    }
}

impl TryFrom<NaiveDateTime> for DateTime {
    type Error = SpanError;
    fn try_from(datetime: NaiveDateTime) -> Result<Self, Self::Error> {
        Self::build(datetime)
    }
}

impl TryFrom<i32> for DateTime {
    type Error = SpanError;
    fn try_from(timestamp: i32) -> Result<Self, Self::Error> {
        let datetime = match chrono::DateTime::from_timestamp(timestamp as i64, 0) {
            Some(datetime) => datetime,
            None => {
                return Err(SpanError::ParseFromTimestamp(
                    "Error while parsing timestamp from i32".to_string(),
                ))
                .err_ctx(DateTimeError);
            }
        };
        Self::build(datetime)
    }
}

impl TryFrom<i64> for DateTime {
    type Error = SpanError;
    fn try_from(timestamp: i64) -> Result<Self, Self::Error> {
        let datetime = match chrono::DateTime::from_timestamp(timestamp, 0) {
            Some(datetime) => datetime,
            None => {
                return Err(SpanError::ParseFromTimestamp(
                    "Error while parsing timestamp from i64".to_string(),
                ))
                .err_ctx(DateTimeError);
            }
        };
        Self::build(datetime)
    }
}

impl TryFrom<(String, String)> for DateTime {
    type Error = SpanError;
    fn try_from((datetime, format): (String, String)) -> Result<Self, Self::Error> {
        Self::new(datetime, format)
    }
}

impl TryFrom<(&str, &str)> for DateTime {
    type Error = SpanError;
    fn try_from((datetime, format): (&str, &str)) -> Result<Self, Self::Error> {
        Self::new(datetime, format)
    }
}

impl TryFrom<&str> for DateTime {
    type Error = SpanError;
    fn try_from(datetime: &str) -> Result<Self, Self::Error> {
        Self::build(datetime)
    }
}

impl TryFrom<chrono::DateTime<Utc>> for DateTime {
    type Error = SpanError;
    fn try_from(value: chrono::DateTime<Utc>) -> Result<Self, Self::Error> {
        value.naive_utc().try_into()
    }
}

impl TryFrom<&DateTime> for chrono::DateTime<Utc> {
    type Error = SpanError;
    fn try_from(value: &DateTime) -> Result<Self, Self::Error> {
        let date = value.datetime;
        match Utc::now()
            .with_year(date.year())
            .and_then(|utc| utc.with_month(date.month()))
            .and_then(|utc| utc.with_day(date.day()))
            .and_then(|utc| utc.with_hour(date.hour()))
            .and_then(|utc| utc.with_minute(date.minute()))
            .and_then(|utc| utc.with_second(date.second()))
        {
            Some(utc) => Ok(utc),
            None => Err(SpanError::InvalidUtc).err_ctx(DateTimeError),
        }
    }
}

#[cfg(test)]
pub mod test {
    use chrono::TimeDelta;

    use super::*;

    #[test]
    fn datetime_add_overflow() -> Result<(), SpanError> {
        let mut datetime = DateTime::build("2023-10-09 00:00:00")?;
        let new_datetime = datetime.update(DateTimeUnit::Day, i32::MAX);
        assert_eq!(
            new_datetime,
            Err(SpanError::InvalidUpdate(
                "Cannot Add/Remove 2147483647 Day to/from 2023-10-09 00:00:00".to_string()
            ))
            .err_ctx(DateTimeError)
        );
        Ok(())
    }

    #[test]
    fn datetime_add_one_year() -> Result<(), SpanError> {
        let mut datetime = DateTime::build("2023-10-09 00:00:00")?;
        let new_datetime = datetime.update(DateTimeUnit::Year, 1);
        assert_eq!(new_datetime, Ok(()));
        assert_eq!(datetime.to_string(), "2024-10-09 00:00:00".to_string());
        Ok(())
    }

    #[test]
    fn datetime_remove_one_year() -> Result<(), SpanError> {
        let mut datetime = DateTime::build("2023-10-09 00:00:00")?;
        let new_datetime = datetime.update(DateTimeUnit::Year, -1);
        assert_eq!(new_datetime, Ok(()));
        assert_eq!(datetime.to_string(), "2022-10-09 00:00:00".to_string());
        Ok(())
    }

    #[test]
    fn datetime_add_one_month() -> Result<(), SpanError> {
        let mut datetime = DateTime::build("2023-10-09 00:00:00")?;
        let new_datetime = datetime.update(DateTimeUnit::Month, 1);
        assert_eq!(new_datetime, Ok(()));
        assert_eq!(datetime.to_string(), "2023-11-09 00:00:00".to_string());
        Ok(())
    }

    #[test]
    fn datetime_remove_one_month() -> Result<(), SpanError> {
        let mut datetime = DateTime::build("2023-10-09 00:00:00")?;
        let new_datetime = datetime.update(DateTimeUnit::Month, -1);
        assert_eq!(new_datetime, Ok(()));
        assert_eq!(datetime.to_string(), "2023-09-09 00:00:00".to_string());
        Ok(())
    }

    #[test]
    fn datetime_add_one_day() -> Result<(), SpanError> {
        let mut datetime = DateTime::build("2023-10-09 00:00:00")?;
        let new_datetime = datetime.update(DateTimeUnit::Day, 1);
        assert_eq!(new_datetime, Ok(()));
        assert_eq!(datetime.to_string(), "2023-10-10 00:00:00".to_string());
        Ok(())
    }

    #[test]
    fn datetime_remove_one_day() -> Result<(), SpanError> {
        let mut datetime = DateTime::build("2023-10-09 00:00:00")?;
        let new_datetime = datetime.update(DateTimeUnit::Day, -1);
        assert_eq!(new_datetime, Ok(()));
        assert_eq!(datetime.to_string(), "2023-10-08 00:00:00".to_string());
        Ok(())
    }

    #[test]
    fn datetime_add_one_hour() -> Result<(), SpanError> {
        let mut datetime = DateTime::build("2023-10-09 00:00:00")?;
        let new_datetime = datetime.update(DateTimeUnit::Hour, 1);
        assert_eq!(new_datetime, Ok(()));
        assert_eq!(datetime.to_string(), "2023-10-09 01:00:00".to_string());
        Ok(())
    }

    #[test]
    fn datetime_remove_one_hour() -> Result<(), SpanError> {
        let mut datetime = DateTime::build("2023-10-09 00:00:00")?;
        let new_datetime = datetime.update(DateTimeUnit::Hour, -1);
        assert_eq!(new_datetime, Ok(()));
        assert_eq!(datetime.to_string(), "2023-10-08 23:00:00".to_string());
        Ok(())
    }

    #[test]
    fn datetime_add_one_minute() -> Result<(), SpanError> {
        let mut datetime = DateTime::build("2023-10-09 00:00:00")?;
        let new_datetime = datetime.update(DateTimeUnit::Minute, 1);
        assert_eq!(new_datetime, Ok(()));
        assert_eq!(datetime.to_string(), "2023-10-09 00:01:00".to_string());
        Ok(())
    }

    #[test]
    fn datetime_remove_one_minute() -> Result<(), SpanError> {
        let mut datetime = DateTime::build("2023-10-09 00:00:00")?;
        let new_datetime = datetime.update(DateTimeUnit::Minute, -1);
        assert_eq!(new_datetime, Ok(()));
        assert_eq!(datetime.to_string(), "2023-10-08 23:59:00".to_string());
        Ok(())
    }

    #[test]
    fn datetime_add_one_second() -> Result<(), SpanError> {
        let mut datetime = DateTime::build("2023-10-09 00:00:00")?;
        let new_datetime = datetime.update(DateTimeUnit::Second, 1);
        assert_eq!(new_datetime, Ok(()));
        assert_eq!(datetime.to_string(), "2023-10-09 00:00:01".to_string());
        Ok(())
    }

    #[test]
    fn datetime_remove_one_second() -> Result<(), SpanError> {
        let mut datetime = DateTime::build("2023-10-09 00:00:00")?;
        let new_datetime = datetime.update(DateTimeUnit::Second, -1);
        assert_eq!(new_datetime, Ok(()));
        assert_eq!(datetime.to_string(), "2023-10-08 23:59:59".to_string());
        Ok(())
    }

    #[test]
    fn datetime_serialize() -> Result<(), SpanError> {
        let datetime = DateTime::build("2023-10-09 00:00:00")?;
        let Ok(serialized) = serde_json::to_string(&datetime) else {
            panic!("Error while serializing datetime");
        };
        assert_eq!(
            serialized,
            "{\"datetime\":\"2023-10-09 00:00:00\",\"format\":\"%Y-%m-%d %H:%M:%S\"}".to_string()
        );
        Ok(())
    }

    #[test]
    fn datetime_deserialize() -> Result<(), SpanError> {
        let serialized =
            "{\"datetime\":\"2023-10-09 00:00:00\",\"format\":\"%Y-%m-%d %H:%M:%S\"}".to_string();
        let Ok(datetime) = serde_json::from_str::<DateTime>(&serialized) else {
            panic!("Error while deserializing datetime");
        };
        assert_eq!(datetime.to_string(), "2023-10-09 00:00:00".to_string());
        assert_eq!(datetime.format, BASE_DATETIME_FORMAT.get().to_string());
        Ok(())
    }

    #[test]
    fn datetime_serialize_format() -> Result<(), SpanError> {
        let datetime = DateTime::build("2023-10-09 00:00:00")?.format("%d/%m/%YT%H_%M_%S");
        let Ok(serialized) = serde_json::to_string(&datetime) else {
            panic!("Error while serializing datetime");
        };
        assert_eq!(
            serialized,
            "{\"datetime\":\"2023-10-09 00:00:00\",\"format\":\"%d/%m/%YT%H_%M_%S\"}".to_string()
        );
        Ok(())
    }

    #[test]
    fn datetime_deserialize_format() -> Result<(), SpanError> {
        let serialized =
            "{\"datetime\":\"2023-10-09 00:00:00\",\"format\":\"%d/%m/%YT%H_%M_%S\"}".to_string();
        let Ok(datetime) = serde_json::from_str::<DateTime>(&serialized) else {
            panic!("Error while deserializing datetime");
        };
        assert_eq!(datetime.to_string(), "09/10/2023T00_00_00".to_string());
        assert_eq!(datetime.format, "%d/%m/%YT%H_%M_%S".to_string());
        Ok(())
    }

    #[test]
    fn next_month() -> Result<(), SpanError> {
        let mut datetime = DateTime::build("2023-10-09 00:00:00")?;
        datetime.next(DateTimeUnit::Month)?;
        assert_eq!(datetime.to_string(), "2023-11-09 00:00:00".to_string());
        Ok(())
    }

    #[test]
    fn next_minute() -> Result<(), SpanError> {
        let mut datetime = DateTime::build("2023-10-09 00:00:00")?;
        datetime.next(DateTimeUnit::Minute)?;
        assert_eq!(datetime.to_string(), "2023-10-09 00:01:00".to_string());
        Ok(())
    }

    #[test]
    fn next_month_on_december() -> Result<(), SpanError> {
        let mut datetime = DateTime::build("2023-12-09 00:00:00")?;
        datetime.next(DateTimeUnit::Month)?;
        assert_eq!(datetime.to_string(), "2024-01-09 00:00:00".to_string());
        Ok(())
    }

    #[test]
    fn next_hour_on_midnight() -> Result<(), SpanError> {
        let mut datetime = DateTime::build("2023-10-09 23:59:34")?;
        datetime.next(DateTimeUnit::Hour)?;
        assert_eq!(datetime.to_string(), "2023-10-10 00:59:34".to_string());
        Ok(())
    }

    #[test]
    fn next_day_28_february_leap_year() -> Result<(), SpanError> {
        let mut datetime = DateTime::build("2024-02-28 00:00:00")?;
        datetime.next(DateTimeUnit::Day)?;
        assert_eq!(datetime.to_string(), "2024-02-29 00:00:00".to_string());
        Ok(())
    }

    #[test]
    fn next_day_28_february_non_leap_year() -> Result<(), SpanError> {
        let mut datetime = DateTime::build("2023-02-28 00:00:00")?;
        datetime.next(DateTimeUnit::Day)?;
        assert_eq!(datetime.to_string(), "2023-03-01 00:00:00".to_string());
        Ok(())
    }

    #[test]
    fn matches_every_unit_in_datetime() -> Result<(), SpanError> {
        let datetime = DateTime::build("2023-10-09 05:23:18")?;
        assert!(datetime.matches(DateTimeUnit::Year, 2023));
        assert!(datetime.matches(DateTimeUnit::Month, 10));
        assert!(datetime.matches(DateTimeUnit::Day, 9));
        assert!(datetime.matches(DateTimeUnit::Hour, 5));
        assert!(datetime.matches(DateTimeUnit::Minute, 23));
        assert!(datetime.matches(DateTimeUnit::Second, 18));
        Ok(())
    }

    #[test]
    fn is_in_future_yesterday() -> Result<(), SpanError> {
        let mut datetime = DateTime::now()?;
        datetime.update(DateTimeUnit::Day, -1)?;
        assert!(!datetime.is_in_future()?);
        Ok(())
    }

    #[test]
    fn is_in_future_tomorrow() -> Result<(), SpanError> {
        let mut datetime = DateTime::now()?;
        datetime.update(DateTimeUnit::Day, 1)?;
        assert!(datetime.is_in_future()?);
        Ok(())
    }

    #[test]
    fn is_in_future_now() -> Result<(), SpanError> {
        let datetime = DateTime::now()?;
        assert!(!datetime.is_in_future()?);
        Ok(())
    }

    #[test]
    fn elapsed_one_year() -> Result<(), SpanError> {
        let datetime = DateTime::build("2023-10-09 00:00:00")?;
        let lhs = DateTime::build("2022-10-09 00:00:00")?;
        assert_eq!(datetime.elapsed(&lhs), TimeDelta::try_days(365).unwrap());
        Ok(())
    }

    #[test]
    fn elapsed_one_second() -> Result<(), SpanError> {
        let datetime = DateTime::build("2023-10-09 00:00:01")?;
        let lhs = DateTime::build("2023-10-09 00:00:00")?;
        assert_eq!(datetime.elapsed(&lhs), TimeDelta::try_seconds(1).unwrap());
        Ok(())
    }

    #[test]
    fn elapsed_multiple_units() -> Result<(), SpanError> {
        let datetime = DateTime::build("2023-10-09 01:01:01")?;
        let lhs = DateTime::build("2023-10-08 00:00:00")?;
        assert_eq!(
            datetime.elapsed(&lhs),
            TimeDelta::try_days(1)
                .unwrap()
                .checked_add(&TimeDelta::try_hours(1).unwrap())
                .unwrap()
                .checked_add(&TimeDelta::try_minutes(1).unwrap())
                .unwrap()
                .checked_add(&TimeDelta::try_seconds(1).unwrap())
                .unwrap()
        );
        Ok(())
    }

    #[test]
    fn unit_in_between() -> Result<(), SpanError> {
        let datetime = DateTime::build("2023-10-09 01:01:01")?;
        let lhs = DateTime::build("2023-10-08 00:00:00")?;
        let years_in_between = datetime.unit_in_between(DateTimeUnit::Year, &lhs);
        let months_in_between = datetime.unit_in_between(DateTimeUnit::Month, &lhs);
        let days_in_between = datetime.unit_in_between(DateTimeUnit::Day, &lhs);
        let hours_in_between = datetime.unit_in_between(DateTimeUnit::Hour, &lhs);
        let minutes_in_between = datetime.unit_in_between(DateTimeUnit::Minute, &lhs);
        let seconds_in_between = datetime.unit_in_between(DateTimeUnit::Second, &lhs);
        assert_eq!(years_in_between, 0);
        assert_eq!(months_in_between, 0);
        assert_eq!(days_in_between, 1);
        assert_eq!(hours_in_between, days_in_between * 24 + 1);
        assert_eq!(minutes_in_between, hours_in_between * 60 + 1);
        assert_eq!(seconds_in_between, minutes_in_between * 60 + 1);
        Ok(())
    }

    #[test]
    fn unit_in_between_leap_year_days() -> Result<(), SpanError> {
        let datetime = DateTime::build("2024-03-12 00:00:00")?;
        let lhs = DateTime::build("2024-01-12 00:00:00")?;
        let years_in_between = datetime.unit_in_between(DateTimeUnit::Year, &lhs);
        let months_in_between = datetime.unit_in_between(DateTimeUnit::Month, &lhs);
        let days_in_between = datetime.unit_in_between(DateTimeUnit::Day, &lhs);
        assert_eq!(years_in_between, 0);
        assert_eq!(months_in_between, years_in_between * 12 + 2);
        assert_eq!(days_in_between, 60);
        Ok(())
    }

    #[test]
    fn unit_in_between_non_leap_year_days() -> Result<(), SpanError> {
        let datetime = DateTime::build("2023-03-12 00:00:00")?;
        let lhs = DateTime::build("2023-01-12 00:00:00")?;
        let years_in_between = datetime.unit_in_between(DateTimeUnit::Year, &lhs);
        let months_in_between = datetime.unit_in_between(DateTimeUnit::Month, &lhs);
        let days_in_between = datetime.unit_in_between(DateTimeUnit::Day, &lhs);
        assert_eq!(years_in_between, 0);
        assert_eq!(months_in_between, years_in_between * 12 + 2);
        assert_eq!(days_in_between, 59);
        Ok(())
    }

    #[test]
    fn clear_time() -> Result<(), SpanError> {
        let datetime = DateTime::build("2023-10-09 01:01:01")?;
        let datetime = datetime.clear_time()?;
        assert_eq!(datetime.to_string(), "2023-10-09 00:00:00".to_string());
        Ok(())
    }
}
