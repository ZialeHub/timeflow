#[cfg(feature = "datetime")]
pub mod datetime {
    use std::{
        ops::{Deref, DerefMut},
        sync::{LazyLock, RwLock},
    };

    use chrono::{Datelike, Days, Duration, Local, Months, NaiveDateTime, Timelike, Utc};
    use serde::{Deserialize, Serialize};

    use crate::{
        error::{DateTimeError, ErrorContext, SpanError},
        span::Span,
        timestamp::{TimestampMicro, TimestampMilli, TimestampNano},
        BaseFormat, GetInner,
    };

    pub(crate) static BASE_DATETIME_FORMAT: BaseFormat<Option<&'static str>> =
        LazyLock::new(|| RwLock::new(None));

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
    #[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Clone, Serialize, Deserialize)]
    pub struct DateTime {
        pub(crate) datetime: NaiveDateTime,
        pub(crate) format: String,
    }

    impl Default for DateTime {
        fn default() -> Self {
            Self {
                datetime: NaiveDateTime::default(),
                format: BASE_DATETIME_FORMAT.get().to_string(),
            }
        }
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
        /// Getter for the datetime
        pub fn datetime(&self) -> NaiveDateTime {
            self.datetime
        }

        /// Return the timestamp from the [DateTime]
        pub fn timestamp(&self) -> i64 {
            self.datetime.and_utc().timestamp()
        }

        /// let datetime = DateTime::new(2023, 10, 09)?.with_time(01, 01, 01)?;
        /// let datetime = datetime.clear_time()?;
        /// assert_eq!(datetime.to_string(), "2023-10-09 00:00:00".to_string());
        /// ```
        pub fn clear_time(&self) -> Self {
            let datetime = NaiveDateTime::new(self.datetime.date(), chrono::NaiveTime::default());
            Self {
                datetime,
                format: BASE_DATETIME_FORMAT.get().to_string(),
            }
        }

        /// Create a new variable [DateTime] from the parameter `datetime` formated with [BASE_DATETIME_FORMAT](static@BASE_DATETIME_FORMAT)
        ///
        /// # Example
        ///
        /// ```rust,ignore
        /// let datetime = DateTime::new(2023, 05, 17)?.with_time(09, 05, 12)?;
        /// ```
        ///
        /// # Errors
        ///
        /// Return an Err(_) if the given `datetime` is not formated with [BASE_DATETIME_FORMAT](static@BASE_DATETIME_FORMAT)
        pub fn with_time(mut self, hour: u32, minute: u32, second: u32) -> Result<Self, SpanError> {
            let Some(time) = chrono::NaiveTime::from_hms_opt(hour, minute, second) else {
                return Err(SpanError::InvalidTime(hour, minute, second)).err_ctx(DateTimeError);
            };
            self.datetime = self.datetime.date().and_time(time);
            Ok(self)
        }
    }

    impl Span<DateTimeUnit, i32> for DateTime {
        /// Create a new variable [DateTime] from year, month and day
        ///
        /// Default time is set to 00:00:00
        ///
        /// Use [BASE_DATETIME_FORMAT](static@BASE_DATETIME_FORMAT) as default format
        ///
        /// # Example
        ///
        /// ```rust,ignore
        /// let datetime = DateTime::new(2024, 09, 27)?;
        /// ```
        ///
        /// # Errors
        ///
        /// Return an Err(_) if `datetime` is not formated with `format`
        fn new(year: i32, month: u32, day: u32) -> Result<Self, SpanError> {
            let date = chrono::NaiveDate::from_ymd_opt(year, month, day)
                .ok_or(SpanError::InvalidDate(year, month, day))
                .err_ctx(DateTimeError)?;
            let datetime = NaiveDateTime::new(date, chrono::NaiveTime::default());
            Ok(Self {
                datetime,
                format: BASE_DATETIME_FORMAT.get().to_string(),
            })
        }

        /// Setter for the format
        ///
        /// None will set the format to [BASE_DATETIME_FORMAT](static@BASE_DATETIME_FORMAT)
        ///
        ///  See the [chrono::format::strftime] for the supported escape sequences of `format`.
        fn format(mut self, format: Option<impl ToString>) -> Self {
            self.format = match format {
                Some(format) => format.to_string(),
                None => BASE_DATETIME_FORMAT.get().to_string(),
            };
            self
        }

        /// Function to increase / decrease the datetime [DateTime] by [DateTimeUnit]
        ///
        /// # Example
        /// ```rust,ignore
        /// let mut datetime = DateTime::new(2023, 10, 09)?;
        ///
        /// datetime.update(DateTimeUnit::Year, 1)?;
        /// assert_eq!(datetime.to_string(), "2024-10-09 00:00:00".to_string());
        ///
        /// datetime.update(DateTimeUnit::Minute, 5)?;
        /// assert_eq!(datetime.to_string(), "2024-10-09 00:05:00".to_string());
        /// ```
        ///
        /// # Errors
        /// The function will return an Err(_) if the operation is not possible or [chrono] fails to update the datetime
        fn update(&self, unit: DateTimeUnit, value: i32) -> Result<Self, SpanError> {
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
                Some(datetime) => Ok(Self {
                    datetime,
                    format: self.format.clone(),
                }),
                None => Err(SpanError::InvalidUpdate(format!(
                    "Cannot Add/Remove {} {:?} to/from {}",
                    value, unit, self
                )))
                .err_ctx(DateTimeError),
            }
        }

        /// Go to the next [DateTimeUnit] from [DateTime]
        ///
        /// # Example
        /// ```rust,ignore
        /// let mut datetime = DateTime::new(2023, 01, 31)?.with_time(12, 09, 27)?;
        /// datetime.next(DateTimeUnit::Month);
        /// assert_eq!(datetime.to_string(), "2023-02-28 12:09:27".to_string());
        ///
        /// let mut datetime = DateTime::new(2023, 10, 09)?;
        /// datetime.next(DateTimeUnit::Month);
        /// assert_eq!(datetime.to_string(), "2023-11-09 00:00:00".to_string());
        /// ```
        ///
        /// # Errors
        /// The function will return an Err(_) if the operation is not possible
        fn next(&self, unit: DateTimeUnit) -> Result<Self, SpanError> {
            self.update(unit, 1)
        }

        /// Compare the [DateTimeUnit] from [DateTime] and value ([u32])
        ///
        /// # Example
        /// ```rust,ignore
        /// let datetime = DateTime::new(2023, 10, 09)?.with_time(05, 23, 18)?;
        /// assert!(datetime.matches(DateTimeUnit::Year, 2023));
        /// assert!(datetime.matches(DateTimeUnit::Month, 10));
        /// assert!(!datetime.matches(DateTimeUnit::Minute, 53));
        /// ```
        fn matches(&self, unit: DateTimeUnit, value: u32) -> bool {
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
        fn now() -> Result<Self, SpanError> {
            let datetime = Local::now();
            Self::new(datetime.year(), datetime.month(), datetime.day())?.with_time(
                datetime.hour(),
                datetime.minute(),
                datetime.second(),
            )
        }

        /// Return a [bool] to know if the [DateTime] is in the future
        ///
        /// # Example
        /// ```rust,ignore
        /// let datetime = DateTime::new(2023, 10, 09)?.with_time(05, 23, 18)?;
        /// // If Now is 2024-01-01 00:00:00
        /// assert!(!datetime.is_in_future()?);
        ///
        /// let datetime = DateTime::new(2025, 01, 09)?.with_time(02, 01, 15)?;
        /// // If Now is 2024-01-01 00:00:00
        /// assert!(datetime.is_in_future()?);
        /// ```
        fn is_in_future(&self) -> Result<bool, SpanError> {
            let datetime = Local::now();
            let now = Self::new(datetime.year(), datetime.month(), datetime.day())?.with_time(
                datetime.hour(),
                datetime.minute(),
                datetime.second(),
            )?;
            Ok(self.datetime > now.datetime)
        }

        /// Elapsed [Duration] between two [DateTime]
        ///
        /// # Example
        /// ```rust,ignore
        /// let rhs = DateTime::new(2023, 10, 09)?;
        /// let lhs = DateTime::new(2022, 10, 09)?;
        /// assert_eq!(rhs.elapsed(&lhs), TimeDelta::try_days(365).unwrap());
        /// ```
        fn elapsed(&self, lhs: &Self) -> Duration {
            self.datetime.signed_duration_since(lhs.datetime)
        }

        /// Number of [DateTimeUnit] between two [DateTime]
        ///
        /// # Example
        /// ```rust,ignore
        /// let lhs = DateTime::new(2023, 10, 09)?.with_time(01, 01, 01)?;
        /// let rhs = DateTime::new(2023, 10, 08)?;
        /// let years_in_between = lhs.unit_elapsed(&rhs, DateTimeUnit::Year);
        /// let months_in_between = lhs.unit_elapsed(&rhs, DateTimeUnit::Month);
        /// let days_in_between = lhs.unit_elapsed(&rhs, DateTimeUnit::Day);
        /// let hours_in_between = lhs.unit_elapsed(&rhs, DateTimeUnit::Hour);
        /// let minutes_in_between = lhs.unit_elapsed(&rhs, DateTimeUnit::Minute);
        /// let seconds_in_between = lhs.unit_elapsed(&rhs, DateTimeUnit::Second);
        /// assert_eq!(years_in_between, 0);
        /// assert_eq!(months_in_between, 0);
        /// assert_eq!(days_in_between, 1);
        /// assert_eq!(hours_in_between, days_in_between * 24 + 1);
        /// assert_eq!(minutes_in_between, hours_in_between * 60 + 1);
        /// assert_eq!(seconds_in_between, minutes_in_between * 60 + 1);
        /// ```
        fn unit_elapsed(&self, rhs: &Self, unit: DateTimeUnit) -> Result<i64, SpanError> {
            Ok(match unit {
                DateTimeUnit::Year => (self.datetime.year() - rhs.datetime.year()) as i64,
                DateTimeUnit::Month => {
                    self.datetime.year() as i64 * 12 + self.datetime.month() as i64
                        - (rhs.datetime.year() as i64 * 12 + rhs.datetime.month() as i64)
                }
                DateTimeUnit::Day => {
                    (self.datetime.and_utc().timestamp() - rhs.datetime.and_utc().timestamp())
                        / 60
                        / 60
                        / 24
                }
                DateTimeUnit::Hour => {
                    (self.datetime.and_utc().timestamp() - rhs.datetime.and_utc().timestamp())
                        / 60
                        / 60
                }
                DateTimeUnit::Minute => {
                    (self.datetime.and_utc().timestamp() - rhs.datetime.and_utc().timestamp()) / 60
                }
                DateTimeUnit::Second => {
                    self.datetime.and_utc().timestamp() - rhs.datetime.and_utc().timestamp()
                }
            }
            .abs())
        }

        /// Clear the [DateTimeUnit] from [DateTime]
        ///
        /// # Errors
        /// Return an Err(_) if the [DateTimeUnit] is not valid
        ///
        /// # Example
        /// ```rust,ignore
        /// let datetime = DateTime::build("2023-05-17 09:05:12")?;
        /// let year = datetime.clear_unit(DateTimeUnit::Year)?;
        /// assert_eq!(year.to_string(), "1970-05-17 09:05:12".to_string());
        /// let month = datetime.clear_unit(DateTimeUnit::Month)?;
        /// assert_eq!(month.to_string(), "2023-01-17 09:05:12".to_string());
        /// let day = datetime.clear_unit(DateTimeUnit::Day)?;
        /// assert_eq!(day.to_string(), "2023-05-01 09:05:12".to_string());
        /// let hour = datetime.clear_unit(DateTimeUnit::Hour)?;
        /// assert_eq!(hour.to_string(), "2023-05-17 00:05:12".to_string());
        /// let minute = datetime.clear_unit(DateTimeUnit::Minute)?;
        /// assert_eq!(minute.to_string(), "2023-05-17 09:00:12".to_string());
        /// let second = datetime.clear_unit(DateTimeUnit::Second)?;
        /// assert_eq!(second.to_string(), "2023-05-17 09:05:00".to_string());
        /// ```
        fn clear_unit(&self, unit: DateTimeUnit) -> Result<Self, SpanError> {
            let datetime = match unit {
                DateTimeUnit::Year => self.datetime.with_year(1970).ok_or(SpanError::ClearUnit(
                    "Error while setting year to 1970".to_string(),
                )),
                DateTimeUnit::Month => self.datetime.with_month(1).ok_or(SpanError::ClearUnit(
                    "Error while setting month to 1".to_string(),
                )),
                DateTimeUnit::Day => self.datetime.with_day(1).ok_or(SpanError::ClearUnit(
                    "Error while setting day to 1".to_string(),
                )),
                DateTimeUnit::Hour => self.datetime.with_hour(0).ok_or(SpanError::ClearUnit(
                    "Error while setting hour to 0".to_string(),
                )),
                DateTimeUnit::Minute => self.datetime.with_minute(0).ok_or(SpanError::ClearUnit(
                    "Error while setting minute to 0".to_string(),
                )),
                DateTimeUnit::Second => self.datetime.with_second(0).ok_or(SpanError::ClearUnit(
                    "Error while setting second to 0".to_string(),
                )),
            }
            .err_ctx(DateTimeError)?;
            Ok(Self {
                datetime,
                format: self.format.clone(),
            })
        }
    }

    impl From<NaiveDateTime> for DateTime {
        fn from(datetime: NaiveDateTime) -> Self {
            Self {
                datetime,
                format: BASE_DATETIME_FORMAT.get().to_string(),
            }
        }
    }

    impl TryFrom<TimestampMilli> for crate::datetime::DateTime {
        type Error = crate::error::SpanError;

        fn try_from(timestamp: TimestampMilli) -> Result<Self, Self::Error> {
            let datetime = chrono::DateTime::from_timestamp(*timestamp, 0).ok_or(
                crate::error::SpanError::ParseFromTimestamp(
                    "Error while parsing timestamp".to_string(),
                ),
            )?;
            Ok(Self {
                datetime: datetime.naive_utc(),
                format: crate::datetime::BASE_DATETIME_FORMAT.get(),
            })
        }
    }

    impl TryFrom<TimestampMicro> for crate::datetime::DateTime {
        type Error = crate::error::SpanError;

        fn try_from(timestamp: TimestampMicro) -> Result<Self, Self::Error> {
            let datetime = chrono::DateTime::from_timestamp(*timestamp / 1_000, 0).ok_or(
                crate::error::SpanError::ParseFromTimestamp(
                    "Error while parsing timestamp".to_string(),
                ),
            )?;
            Ok(Self {
                datetime: datetime.naive_utc(),
                format: crate::datetime::BASE_DATETIME_FORMAT.get(),
            })
        }
    }

    impl TryFrom<TimestampNano> for crate::datetime::DateTime {
        type Error = crate::error::SpanError;

        fn try_from(timestamp: TimestampNano) -> Result<Self, Self::Error> {
            let datetime = chrono::DateTime::from_timestamp(*timestamp / 1_000_000, 0).ok_or(
                crate::error::SpanError::ParseFromTimestamp(
                    "Error while parsing timestamp".to_string(),
                ),
            )?;
            Ok(Self {
                datetime: datetime.naive_utc(),
                format: crate::datetime::BASE_DATETIME_FORMAT.get(),
            })
        }
    }

    impl TryFrom<(String, String)> for DateTime {
        type Error = SpanError;
        fn try_from((datetime, format): (String, String)) -> Result<Self, Self::Error> {
            let datetime = chrono::NaiveDateTime::parse_from_str(&datetime, &format)
                .map_err(SpanError::ParseFromStr)
                .err_ctx(DateTimeError)?;
            Ok(Self { datetime, format })
        }
    }

    impl TryFrom<(&str, &str)> for DateTime {
        type Error = SpanError;
        fn try_from((datetime, format): (&str, &str)) -> Result<Self, Self::Error> {
            let datetime = chrono::NaiveDateTime::parse_from_str(datetime, format)
                .map_err(SpanError::ParseFromStr)
                .err_ctx(DateTimeError)?;
            Ok(Self {
                datetime,
                format: format.to_string(),
            })
        }
    }

    impl TryFrom<&str> for DateTime {
        type Error = SpanError;
        fn try_from(datetime: &str) -> Result<Self, Self::Error> {
            let datetime =
                chrono::NaiveDateTime::parse_from_str(datetime, &BASE_DATETIME_FORMAT.get())
                    .map_err(SpanError::ParseFromStr)
                    .err_ctx(DateTimeError)?;
            Ok(Self {
                datetime,
                format: BASE_DATETIME_FORMAT.get().to_string(),
            })
        }
    }

    impl From<chrono::DateTime<Utc>> for DateTime {
        fn from(value: chrono::DateTime<Utc>) -> Self {
            Self::from(value.naive_utc())
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
            let datetime = DateTime::new(2023, 10, 09)?;
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
            let datetime = DateTime::new(2023, 10, 09)?;
            let new_datetime = datetime.update(DateTimeUnit::Year, 1)?;
            assert_eq!(new_datetime.to_string(), "2024-10-09 00:00:00".to_string());
            Ok(())
        }

        #[test]
        fn datetime_remove_one_year() -> Result<(), SpanError> {
            let datetime = DateTime::new(2023, 10, 09)?;
            let new_datetime = datetime.update(DateTimeUnit::Year, -1)?;
            assert_eq!(new_datetime.to_string(), "2022-10-09 00:00:00".to_string());
            Ok(())
        }

        #[test]
        fn datetime_add_one_month() -> Result<(), SpanError> {
            let datetime = DateTime::new(2023, 10, 09)?;
            let new_datetime = datetime.update(DateTimeUnit::Month, 1)?;
            assert_eq!(new_datetime.to_string(), "2023-11-09 00:00:00".to_string());
            Ok(())
        }

        #[test]
        fn datetime_remove_one_month() -> Result<(), SpanError> {
            let datetime = DateTime::new(2023, 10, 09)?;
            let new_datetime = datetime.update(DateTimeUnit::Month, -1)?;
            assert_eq!(new_datetime.to_string(), "2023-09-09 00:00:00".to_string());
            Ok(())
        }

        #[test]
        fn datetime_add_one_day() -> Result<(), SpanError> {
            let datetime = DateTime::new(2023, 10, 09)?;
            let new_datetime = datetime.update(DateTimeUnit::Day, 1)?;
            assert_eq!(new_datetime.to_string(), "2023-10-10 00:00:00".to_string());
            Ok(())
        }

        #[test]
        fn datetime_remove_one_day() -> Result<(), SpanError> {
            let datetime = DateTime::new(2023, 10, 09)?;
            let new_datetime = datetime.update(DateTimeUnit::Day, -1)?;
            assert_eq!(new_datetime.to_string(), "2023-10-08 00:00:00".to_string());
            Ok(())
        }

        #[test]
        fn datetime_add_one_hour() -> Result<(), SpanError> {
            let datetime = DateTime::new(2023, 10, 09)?;
            let new_datetime = datetime.update(DateTimeUnit::Hour, 1)?;
            assert_eq!(new_datetime.to_string(), "2023-10-09 01:00:00".to_string());
            Ok(())
        }

        #[test]
        fn datetime_remove_one_hour() -> Result<(), SpanError> {
            let datetime = DateTime::new(2023, 10, 09)?;
            let new_datetime = datetime.update(DateTimeUnit::Hour, -1)?;
            assert_eq!(new_datetime.to_string(), "2023-10-08 23:00:00".to_string());
            Ok(())
        }

        #[test]
        fn datetime_add_one_minute() -> Result<(), SpanError> {
            let datetime = DateTime::new(2023, 10, 09)?;
            let new_datetime = datetime.update(DateTimeUnit::Minute, 1)?;
            assert_eq!(new_datetime.to_string(), "2023-10-09 00:01:00".to_string());
            Ok(())
        }

        #[test]
        fn datetime_remove_one_minute() -> Result<(), SpanError> {
            let datetime = DateTime::new(2023, 10, 09)?;
            let new_datetime = datetime.update(DateTimeUnit::Minute, -1)?;
            assert_eq!(new_datetime.to_string(), "2023-10-08 23:59:00".to_string());
            Ok(())
        }

        #[test]
        fn datetime_add_one_second() -> Result<(), SpanError> {
            let datetime = DateTime::new(2023, 10, 09)?;
            let new_datetime = datetime.update(DateTimeUnit::Second, 1)?;
            assert_eq!(new_datetime.to_string(), "2023-10-09 00:00:01".to_string());
            Ok(())
        }

        #[test]
        fn datetime_remove_one_second() -> Result<(), SpanError> {
            let datetime = DateTime::new(2023, 10, 09)?;
            let new_datetime = datetime.update(DateTimeUnit::Second, -1)?;
            assert_eq!(new_datetime.to_string(), "2023-10-08 23:59:59".to_string());
            Ok(())
        }

        #[test]
        fn datetime_serialize() -> Result<(), SpanError> {
            let datetime = DateTime::new(2023, 10, 09)?;
            let Ok(serialized) = serde_json::to_string(&datetime) else {
                panic!("Error while serializing datetime");
            };
            assert_eq!(
                serialized,
                "{\"datetime\":\"2023-10-09T00:00:00\",\"format\":\"%Y-%m-%d %H:%M:%S\"}"
                    .to_string()
            );
            Ok(())
        }

        #[test]
        fn datetime_deserialize() -> Result<(), SpanError> {
            let serialized =
                "{\"datetime\":\"2023-10-09T00:00:00\",\"format\":\"%Y-%m-%d %H:%M:%S\"}"
                    .to_string();
            let Ok(datetime) = serde_json::from_str::<DateTime>(&serialized) else {
                panic!("Error while deserializing datetime");
            };
            assert_eq!(datetime.to_string(), "2023-10-09 00:00:00".to_string());
            assert_eq!(datetime.format, BASE_DATETIME_FORMAT.get().to_string());
            Ok(())
        }

        #[test]
        fn datetime_serialize_format() -> Result<(), SpanError> {
            let datetime = DateTime::new(2023, 10, 09)?.format(Some("%d/%m/%YT%H_%M_%S"));
            let Ok(serialized) = serde_json::to_string(&datetime) else {
                panic!("Error while serializing datetime");
            };
            assert_eq!(
                serialized,
                "{\"datetime\":\"2023-10-09T00:00:00\",\"format\":\"%d/%m/%YT%H_%M_%S\"}"
                    .to_string()
            );
            Ok(())
        }

        #[test]
        fn datetime_deserialize_format() -> Result<(), SpanError> {
            let serialized =
                "{\"datetime\":\"2023-10-09T00:00:00\",\"format\":\"%d/%m/%YT%H_%M_%S\"}"
                    .to_string();
            let Ok(datetime) = serde_json::from_str::<DateTime>(&serialized) else {
                panic!("Error while deserializing datetime");
            };
            assert_eq!(datetime.to_string(), "09/10/2023T00_00_00".to_string());
            assert_eq!(datetime.format, "%d/%m/%YT%H_%M_%S".to_string());
            Ok(())
        }

        #[test]
        fn datetime_serialize_in_struct() -> Result<(), SpanError> {
            #[derive(Serialize)]
            struct Test {
                begin_at: DateTime,
            }
            let test = Test {
                begin_at: DateTime::new(2023, 10, 09)?,
            };
            let Ok(serialized) = serde_json::to_string(&test) else {
                panic!("Error while serializing datetime");
            };
            assert_eq!(
            serialized,
            "{\"begin_at\":{\"datetime\":\"2023-10-09T00:00:00\",\"format\":\"%Y-%m-%d %H:%M:%S\"}}".to_string()
        );
            Ok(())
        }

        #[test]
        fn datetime_deserialize_in_struct() -> Result<(), SpanError> {
            #[derive(Deserialize)]
            struct Test {
                begin_at: DateTime,
            }
            let serialized =
            "{\"begin_at\":{\"datetime\":\"2023-10-09T00:00:00\",\"format\":\"%Y-%m-%d %H:%M:%S\"}}".to_string();
            let Ok(test) = serde_json::from_str::<Test>(&serialized) else {
                panic!("Error while deserializing datetime");
            };
            assert_eq!(test.begin_at.to_string(), "2023-10-09 00:00:00".to_string());
            assert_eq!(test.begin_at.format, BASE_DATETIME_FORMAT.get().to_string());
            Ok(())
        }

        #[test]
        fn next_month_january_to_february() -> Result<(), SpanError> {
            let mut datetime = DateTime::new(2023, 01, 31)?.with_time(12, 09, 27)?;
            datetime = datetime.next(DateTimeUnit::Month)?;
            assert_eq!(datetime.to_string(), "2023-02-28 12:09:27".to_string());
            Ok(())
        }

        #[test]
        fn next_month_february_to_march() -> Result<(), SpanError> {
            let mut datetime = DateTime::new(2023, 02, 28)?.with_time(12, 09, 27)?;
            datetime = datetime.next(DateTimeUnit::Month)?;
            assert_eq!(datetime.to_string(), "2023-03-28 12:09:27".to_string());
            Ok(())
        }

        #[test]
        fn next_month() -> Result<(), SpanError> {
            let mut datetime = DateTime::new(2023, 10, 09)?;
            datetime = datetime.next(DateTimeUnit::Month)?;
            assert_eq!(datetime.to_string(), "2023-11-09 00:00:00".to_string());
            Ok(())
        }

        #[test]
        fn next_minute() -> Result<(), SpanError> {
            let mut datetime = DateTime::new(2023, 10, 09)?;
            datetime = datetime.next(DateTimeUnit::Minute)?;
            assert_eq!(datetime.to_string(), "2023-10-09 00:01:00".to_string());
            Ok(())
        }

        #[test]
        fn next_month_on_december() -> Result<(), SpanError> {
            let mut datetime = DateTime::new(2023, 12, 09)?;
            datetime = datetime.next(DateTimeUnit::Month)?;
            assert_eq!(datetime.to_string(), "2024-01-09 00:00:00".to_string());
            Ok(())
        }

        #[test]
        fn next_hour_on_midnight() -> Result<(), SpanError> {
            let mut datetime = DateTime::new(2023, 10, 09)?.with_time(23, 59, 34)?;
            datetime = datetime.next(DateTimeUnit::Hour)?;
            assert_eq!(datetime.to_string(), "2023-10-10 00:59:34".to_string());
            Ok(())
        }

        #[test]
        fn next_day_28_february_leap_year() -> Result<(), SpanError> {
            let mut datetime = DateTime::new(2024, 02, 28)?;
            datetime = datetime.next(DateTimeUnit::Day)?;
            assert_eq!(datetime.to_string(), "2024-02-29 00:00:00".to_string());
            Ok(())
        }

        #[test]
        fn next_day_28_february_non_leap_year() -> Result<(), SpanError> {
            let mut datetime = DateTime::new(2023, 02, 28)?;
            datetime = datetime.next(DateTimeUnit::Day)?;
            assert_eq!(datetime.to_string(), "2023-03-01 00:00:00".to_string());
            Ok(())
        }

        #[test]
        fn matches_every_unit_in_datetime() -> Result<(), SpanError> {
            let datetime = DateTime::new(2023, 10, 09)?.with_time(05, 23, 18)?;
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
            datetime = datetime.update(DateTimeUnit::Day, -1)?;
            assert!(!datetime.is_in_future()?);
            Ok(())
        }

        #[test]
        fn is_in_future_tomorrow() -> Result<(), SpanError> {
            let mut datetime = DateTime::now()?;
            datetime = datetime.update(DateTimeUnit::Day, 1)?;
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
            let datetime = DateTime::new(2023, 10, 09)?;
            let lhs = DateTime::new(2022, 10, 09)?;
            assert_eq!(datetime.elapsed(&lhs), TimeDelta::try_days(365).unwrap());
            Ok(())
        }

        #[test]
        fn elapsed_one_second() -> Result<(), SpanError> {
            let datetime = DateTime::new(2023, 10, 09)?.with_time(00, 00, 01)?;
            let lhs = DateTime::new(2023, 10, 09)?.with_time(00, 00, 00)?;
            assert_eq!(datetime.elapsed(&lhs), TimeDelta::try_seconds(1).unwrap());
            Ok(())
        }

        #[test]
        fn elapsed_multiple_units() -> Result<(), SpanError> {
            let datetime = DateTime::new(2023, 10, 09)?.with_time(01, 01, 01)?;
            let lhs = DateTime::new(2023, 10, 08)?;
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
        fn unit_elapsed() -> Result<(), SpanError> {
            let datetime = DateTime::new(2023, 10, 09)?.with_time(01, 01, 01)?;
            let rhs = DateTime::new(2023, 10, 08)?.with_time(00, 00, 00)?;
            let years_in_between = datetime.unit_elapsed(&rhs, DateTimeUnit::Year)?;
            let months_in_between = datetime.unit_elapsed(&rhs, DateTimeUnit::Month)?;
            let days_in_between = datetime.unit_elapsed(&rhs, DateTimeUnit::Day)?;
            let hours_in_between = datetime.unit_elapsed(&rhs, DateTimeUnit::Hour)?;
            let minutes_in_between = datetime.unit_elapsed(&rhs, DateTimeUnit::Minute)?;
            let seconds_in_between = datetime.unit_elapsed(&rhs, DateTimeUnit::Second)?;
            assert_eq!(years_in_between, 0);
            assert_eq!(months_in_between, 0);
            assert_eq!(days_in_between, 1);
            assert_eq!(hours_in_between, days_in_between * 24 + 1);
            assert_eq!(minutes_in_between, hours_in_between * 60 + 1);
            assert_eq!(seconds_in_between, minutes_in_between * 60 + 1);
            Ok(())
        }

        #[test]
        fn unit_elapsed_leap_year_days() -> Result<(), SpanError> {
            let datetime = DateTime::new(2024, 03, 12)?;
            let rhs = DateTime::new(2024, 01, 12)?;
            let years_in_between = datetime.unit_elapsed(&rhs, DateTimeUnit::Year)?;
            let months_in_between = datetime.unit_elapsed(&rhs, DateTimeUnit::Month)?;
            let days_in_between = datetime.unit_elapsed(&rhs, DateTimeUnit::Day)?;
            assert_eq!(years_in_between, 0);
            assert_eq!(months_in_between, years_in_between * 12 + 2);
            assert_eq!(days_in_between, 60);
            Ok(())
        }

        #[test]
        fn unit_elapsed_non_leap_year_days() -> Result<(), SpanError> {
            let datetime = DateTime::new(2023, 03, 12)?;
            let rhs = DateTime::new(2023, 01, 12)?;
            let years_in_between = datetime.unit_elapsed(&rhs, DateTimeUnit::Year)?;
            let months_in_between = datetime.unit_elapsed(&rhs, DateTimeUnit::Month)?;
            let days_in_between = datetime.unit_elapsed(&rhs, DateTimeUnit::Day)?;
            assert_eq!(years_in_between, 0);
            assert_eq!(months_in_between, years_in_between * 12 + 2);
            assert_eq!(days_in_between, 59);
            Ok(())
        }

        #[test]
        fn clear_time() -> Result<(), SpanError> {
            let datetime = DateTime::new(2023, 10, 09)?.with_time(01, 01, 01)?;
            let datetime = datetime.clear_time();
            assert_eq!(datetime.to_string(), "2023-10-09 00:00:00".to_string());
            Ok(())
        }

        #[test]
        fn datetime_from_timestamp_milli() -> Result<(), SpanError> {
            let timestamp: TimestampMilli = 1735683010.into();
            let datetime: DateTime = DateTime::try_from(timestamp)?;
            assert_eq!(datetime.to_string(), "2024-12-31 22:10:10");
            Ok(())
        }

        #[test]
        fn datetime_from_timestamp_micro() -> Result<(), SpanError> {
            let timestamp: TimestampMicro = 1735683010000.into();
            let datetime: DateTime = DateTime::try_from(timestamp)?;
            assert_eq!(datetime.to_string(), "2024-12-31 22:10:10");
            Ok(())
        }

        #[test]
        fn datetime_from_timestamp_nano() -> Result<(), SpanError> {
            let timestamp: TimestampNano = 1735683010000000.into();
            let datetime: DateTime = DateTime::try_from(timestamp)?;
            assert_eq!(datetime.to_string(), "2024-12-31 22:10:10");
            Ok(())
        }

        #[test]
        fn timestamp_milli_into_datetime() -> Result<(), SpanError> {
            let timestamp: TimestampMilli = 1735683010.into();
            let datetime: DateTime = timestamp.try_into()?;
            assert_eq!(datetime.to_string(), "2024-12-31 22:10:10");
            Ok(())
        }

        #[test]
        fn timestamp_micro_into_datetime() -> Result<(), SpanError> {
            let timestamp: TimestampMicro = 1735683010000.into();
            let datetime: DateTime = timestamp.try_into()?;
            assert_eq!(datetime.to_string(), "2024-12-31 22:10:10");
            Ok(())
        }

        #[test]
        fn timestamp_nano_into_datetime() -> Result<(), SpanError> {
            let timestamp: TimestampNano = 1735683010000000.into();
            let datetime: DateTime = timestamp.try_into()?;
            assert_eq!(datetime.to_string(), "2024-12-31 22:10:10");
            Ok(())
        }

        #[test]
        fn i64_into_datetime() -> Result<(), SpanError> {
            let datetime: DateTime = TimestampMilli::from(1735683010).try_into()?;
            assert_eq!(datetime.to_string(), "2024-12-31 22:10:10");
            Ok(())
        }
    }
}

#[cfg(all(feature = "date", feature = "datetime"))]
mod date_into_datetime {
    use crate::GetInner;

    /// Convert a [Date] to a [DateTime]
    ///
    /// Time will be set to 00:00:00
    ///
    /// # Example
    /// ```rust,ignore
    /// let date = crate::date::Date::new(2023, 10, 09)?;
    /// let datetime = crate::datetime::DateTime::try_from(date)?;
    /// assert_eq!(datetime.to_string(), "2023-10-09 00:00:00".to_string());
    /// ```
    impl From<crate::date::Date> for crate::datetime::DateTime {
        fn from(value: crate::date::Date) -> Self {
            let datetime = chrono::NaiveDateTime::new(value.date(), chrono::NaiveTime::default());
            Self {
                datetime,
                format: crate::datetime::BASE_DATETIME_FORMAT.get().to_string(),
            }
        }
    }

    #[cfg(test)]
    mod test {
        use crate::span::Span;

        #[test]
        fn date_into_datetime() -> Result<(), crate::error::SpanError> {
            let date = crate::date::Date::new(2023, 10, 09)?;
            let datetime = crate::datetime::DateTime::from(date);
            assert_eq!(datetime.to_string(), "2023-10-09 00:00:00".to_string());
            Ok(())
        }

        #[test]
        #[ignore]
        fn date_into_datetime_wrong_format() -> Result<(), crate::error::SpanError> {
            let _span_builder = crate::builder::SpanBuilder::builder()
                .datetime_format("%Y-%m-%d %H:%M:%S")
                .date_format("%d/%m/%Y")
                .build();
            let date = crate::date::Date::new(2023, 10, 09)?;
            let datetime = crate::datetime::DateTime::from(date);
            assert_eq!(datetime.to_string(), "2023-10-09 00:00:00".to_string());
            Ok(())
        }
    }
}

#[cfg(all(feature = "time", feature = "datetime"))]
mod time_into_datetime {
    use crate::GetInner;

    /// Convert a [Time] to a [DateTime]
    ///
    /// Date will be set to 1970-01-01
    ///
    /// # Example
    /// ```rust,ignore
    /// let time = crate::time::Time::new(13, 27, 57)?;
    /// let datetime = crate::datetime::DateTime::try_from(time)?;
    /// assert_eq!(datetime.to_string(), "1970-01-01 13:27:57".to_string());
    /// ```
    impl From<crate::time::Time> for crate::datetime::DateTime {
        fn from(value: crate::time::Time) -> Self {
            let datetime = chrono::NaiveDateTime::new(chrono::NaiveDate::default(), value.time());
            Self {
                datetime,
                format: crate::datetime::BASE_DATETIME_FORMAT.get().to_string(),
            }
        }
    }

    #[cfg(test)]
    mod test {
        use crate::span::Span;

        #[test]
        fn time_into_datetime() -> Result<(), crate::error::SpanError> {
            let time = crate::time::Time::new(13, 27, 57)?;
            let datetime = crate::datetime::DateTime::from(time);
            assert_eq!(datetime.to_string(), "1970-01-01 13:27:57".to_string());
            Ok(())
        }

        #[test]
        #[ignore]
        fn time_into_datetime_wrong_format() -> Result<(), crate::error::SpanError> {
            let _span_builder = crate::builder::SpanBuilder::builder()
                .datetime_format("%Y-%m-%d %H_%M_%S")
                .time_format("%H:%M:%S")
                .build();
            let time = crate::time::Time::new(13, 27, 57)?;
            let datetime = crate::datetime::DateTime::from(time);
            assert_eq!(datetime.to_string(), "1970-01-01 13_27_57".to_string());
            Ok(())
        }
    }
}
