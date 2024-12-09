#[cfg(feature = "time")]
pub mod time {
    use std::{
        ops::{Deref, DerefMut},
        sync::{LazyLock, RwLock},
    };

    use chrono::{Local, NaiveDateTime, NaiveTime, TimeDelta, Timelike, Utc};
    use serde::{Deserialize, Serialize};

    use crate::{
        error::{ErrorContext, SpanError, TimeError},
        span::Span,
        BaseFormat, GetInner,
    };

    pub(crate) static BASE_TIME_FORMAT: BaseFormat<&'static str> =
        LazyLock::new(|| RwLock::new("%H:%M:%S"));

    /// Unit to update [Time]
    #[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Clone, Copy, Serialize, Deserialize)]
    pub enum TimeUnit {
        Hour,
        Minute,
        Second,
    }

    /// Structure to handle time management
    ///
    /// Use [BASE_TIME_FORMAT](static@BASE_TIME_FORMAT) as default format for time
    #[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Clone, Serialize, Deserialize)]
    pub struct Time {
        pub(crate) time: NaiveTime,
        pub(crate) format: String,
    }

    impl Default for Time {
        fn default() -> Self {
            Self::midnight()
        }
    }

    impl std::fmt::Display for Time {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.time.format(&self.format))
        }
    }

    impl Deref for Time {
        type Target = NaiveTime;

        fn deref(&self) -> &Self::Target {
            &self.time
        }
    }

    impl DerefMut for Time {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.time
        }
    }

    impl Time {
        /// Getter for the time
        pub fn time(&self) -> NaiveTime {
            self.time
        }

        /// Return midnight [Time]
        ///
        /// # Example
        ///
        /// ```rust,ignore
        /// assert_eq!(Time::new(00, 00, 00), Time::midnight())
        /// assert_eq!(Time::new(00, 00, 00), Time::default())
        /// ```
        pub fn midnight() -> Self {
            let time = NaiveTime::from_hms_opt(0, 0, 0).expect("Error Time midnight");
            Self {
                time,
                format: BASE_TIME_FORMAT.get().to_string(),
            }
        }
    }

    impl Span<TimeUnit, u32> for Time {
        /// Create a new variable [Time] from hour, minute and second
        ///
        /// Use the format [BASE_TIME_FORMAT](static@BASE_TIME_FORMAT) by default
        ///
        /// # Example
        ///
        /// ```rust,ignore
        /// let time = Time::new(09, 27, 00)?;
        ///
        /// assert_eq!(time.to_string(), "09:27:00".to_string());
        /// ```
        ///
        /// # Errors
        ///
        /// Return an Err(_) if `time` is not formated with `format`
        fn new(hour: u32, minute: u32, second: u32) -> Result<Self, SpanError> {
            let Some(time) = NaiveTime::from_hms_opt(hour, minute, second) else {
                return Err(SpanError::InvalidTime(hour, minute, second)).err_ctx(TimeError);
            };
            Ok(Self {
                time,
                format: BASE_TIME_FORMAT.get().to_string(),
            })
        }

        /// Setter for the format
        ///
        ///  See the [chrono::format::strftime] for the supported escape sequences of `format`.
        fn format(mut self, format: impl ToString) -> Self {
            self.format = format.to_string();
            self
        }

        /// Set the format to [BASE_TIME_FORMAT](static@BASE_TIME_FORMAT)
        fn default_format(mut self) -> Self {
            self.format = BASE_TIME_FORMAT.get().to_string();
            self
        }

        /// Function to increase / decrease the time [Time] by [TimeUnit]
        ///
        /// # Example
        /// ```rust,ignore
        /// let mut time = Time::new(00, 00, 00)?;
        /// time.update(TimeUnit::Hour, 1);
        /// time.update(TimeUnit::Minute, 4);
        /// time.update(TimeUnit::Second, 30);
        /// assert_eq!(time.to_string(), "01:04:30".to_string());
        /// ```
        ///
        /// # Errors
        /// Return an Err(_) if the operation is not possible
        fn update(&self, unit: TimeUnit, value: i32) -> Result<Self, SpanError> {
            let delta_time = match unit {
                TimeUnit::Hour => TimeDelta::new(value as i64 * 60 * 60, 0),
                TimeUnit::Minute => TimeDelta::new(value as i64 * 60, 0),
                TimeUnit::Second => TimeDelta::new(value as i64, 0),
            };
            match delta_time {
                Some(delta_time) => Ok(Self {
                    time: self.time + delta_time,
                    format: self.format.clone(),
                }),
                None => Err(SpanError::InvalidUpdate(format!(
                    "Cannot Add/Remove {} {:?} to/from {}",
                    value, unit, self
                )))
                .err_ctx(TimeError),
            }
        }

        /// Go to the next [TimeUnit] from [Time]
        ///
        /// # Example
        /// ```rust,ignore
        /// let mut time = Time::new(00, 00, 00)?;
        /// time.next(TimeUnit::Hour);
        /// assert_eq!(time.to_string(), "01:00:00".to_string());
        /// ```
        fn next(&self, unit: TimeUnit) -> Result<Self, SpanError> {
            self.update(unit, 1)
        }

        /// Compare the [TimeUnit] from [Time] and value ([u32])
        ///
        /// # Example
        /// ```rust,ignore
        /// let time = Time::new(06, 32, 05)?;
        /// assert!(time.matches(TimeUnit::Hour, 6));
        /// assert!(time.matches(TimeUnit::Minute, 32));
        /// assert!(time.matches(TimeUnit::Second, 5));
        /// ```
        fn matches(&self, unit: TimeUnit, value: u32) -> bool {
            match unit {
                TimeUnit::Hour => self.time.hour() == value,
                TimeUnit::Minute => self.time.minute() == value,
                TimeUnit::Second => self.time.second() == value,
            }
        }

        /// Return the current [Time] from the system
        fn now() -> Result<Self, SpanError> {
            let time = Local::now();
            Self::new(time.hour(), time.minute(), time.second())
        }

        /// Elapsed [TimeDelta] between two [Time]
        ///
        /// # Example
        /// ```rust,ignore
        /// let rhs = Time::new(00, 03, 00)?;
        /// let lhs = Time::new(00, 00, 00)?;
        /// assert_eq!(rhs.elapsed(&lhs), TimeDelta::try_minutes(3).unwrap());
        /// ```
        fn elapsed(&self, lhs: &Self) -> TimeDelta {
            self.time.signed_duration_since(lhs.time)
        }

        /// Number of [TimeUnit] between two [Time]
        ///
        /// # Example
        /// ```rust,ignore
        /// let lhs = Time::new(01, 34, 45)?;
        /// let rhs = Time::new(00, 00, 00)?;
        /// assert_eq!(lhs.unit_elapsed(&rhs, TimeUnit::Hour), 1);
        /// assert_eq!(lhs.unit_elapsed(&rhs, TimeUnit::Minute), 94);
        /// assert_eq!(lhs.unit_elapsed(&rhs, TimeUnit::Second), 5685);
        /// ```
        fn unit_elapsed(&self, rhs: &Self, unit: TimeUnit) -> Result<i64, SpanError> {
            Ok(match unit {
                TimeUnit::Hour => self.time.signed_duration_since(rhs.time).num_hours(),
                TimeUnit::Minute => self.time.signed_duration_since(rhs.time).num_minutes(),
                TimeUnit::Second => self.time.signed_duration_since(rhs.time).num_seconds(),
            })
        }

        fn is_in_future(&self) -> Result<bool, SpanError> {
            let now = Self::now()?;
            Ok(self.time > now.time)
        }

        /// Clear the [TimeUnit] from [Time]
        ///
        /// # Errors
        /// Return an Err(_) if the [TimeUnit] cannot be cleared
        ///
        /// # Example
        /// ```rust,ignore
        /// let time = Time::build("12:21:46")?;
        /// let hour = time.clear_unit(TimeUnit::Hour)?;
        /// assert_eq!(hour.to_string(), "00:21:46".to_string());
        /// let minute = time.clear_unit(TimeUnit::Minute)?;
        /// assert_eq!(minute.to_string(), "12:00:46".to_string());
        /// let second = time.clear_unit(TimeUnit::Second)?;
        /// assert_eq!(second.to_string(), "12:21:00".to_string());
        /// ```
        fn clear_unit(&self, unit: TimeUnit) -> Result<Self, SpanError> {
            let time = match unit {
                TimeUnit::Hour => self.time.with_hour(0).ok_or(SpanError::ClearUnit(
                    "Error while setting hour to 0".to_string(),
                )),
                TimeUnit::Minute => self.time.with_minute(0).ok_or(SpanError::ClearUnit(
                    "Error while setting minute to 0".to_string(),
                )),
                TimeUnit::Second => self.time.with_second(0).ok_or(SpanError::ClearUnit(
                    "Error while setting second to 0".to_string(),
                )),
            }
            .err_ctx(TimeError)?;
            Ok(Self {
                time,
                format: self.format.clone(),
            })
        }
    }

    impl From<NaiveDateTime> for Time {
        fn from(datetime: NaiveDateTime) -> Self {
            Self {
                time: datetime.time(),
                format: BASE_TIME_FORMAT.get().to_string(),
            }
        }
    }

    impl From<NaiveTime> for Time {
        fn from(time: NaiveTime) -> Self {
            Self {
                time,
                format: BASE_TIME_FORMAT.get().to_string(),
            }
        }
    }

    impl TryFrom<(String, String)> for Time {
        type Error = SpanError;

        fn try_from((time, format): (String, String)) -> Result<Self, Self::Error> {
            let time = NaiveTime::parse_from_str(&time, &format)
                .map_err(SpanError::ParseFromStr)
                .err_ctx(TimeError)?;
            Ok(Self { time, format })
        }
    }

    impl TryFrom<(&str, &str)> for Time {
        type Error = SpanError;

        fn try_from((time, format): (&str, &str)) -> Result<Self, Self::Error> {
            let time = NaiveTime::parse_from_str(time, format)
                .map_err(SpanError::ParseFromStr)
                .err_ctx(TimeError)?;
            Ok(Self {
                time,
                format: format.to_string(),
            })
        }
    }

    impl TryFrom<String> for Time {
        type Error = SpanError;

        fn try_from(time: String) -> Result<Self, Self::Error> {
            let time = NaiveTime::parse_from_str(&time, BASE_TIME_FORMAT.get())
                .map_err(SpanError::ParseFromStr)
                .err_ctx(TimeError)?;
            Self::new(time.hour(), time.minute(), time.second())
        }
    }

    impl TryFrom<&str> for Time {
        type Error = SpanError;

        fn try_from(time: &str) -> Result<Self, Self::Error> {
            let time = NaiveTime::parse_from_str(time, BASE_TIME_FORMAT.get())
                .map_err(SpanError::ParseFromStr)
                .err_ctx(TimeError)?;
            Self::new(time.hour(), time.minute(), time.second())
        }
    }

    impl TryFrom<chrono::DateTime<Utc>> for Time {
        type Error = SpanError;
        fn try_from(value: chrono::DateTime<Utc>) -> Result<Self, Self::Error> {
            Ok(value.naive_utc().into())
        }
    }

    impl TryFrom<&Time> for chrono::DateTime<Utc> {
        type Error = SpanError;
        fn try_from(value: &Time) -> Result<Self, Self::Error> {
            let date = value.time;
            match Utc::now()
                .with_hour(date.hour())
                .and_then(|utc| utc.with_minute(date.minute()))
                .and_then(|utc| utc.with_second(date.second()))
            {
                Some(utc) => Ok(utc),
                None => Err(SpanError::InvalidUtc).err_ctx(TimeError),
            }
        }
    }

    #[cfg(test)]
    pub mod test {
        use super::*;

        #[test]
        fn time_add_overflow() -> Result<(), SpanError> {
            let time = Time::new(00, 00, 00)?;
            let new_time = time.update(TimeUnit::Hour, i32::MIN)?;
            assert_eq!(new_time.to_string(), "16:00:00".to_string());
            Ok(())
        }

        #[test]
        fn time_add_one_hour() -> Result<(), SpanError> {
            let time = Time::new(00, 00, 00)?;
            let new_time = time.update(TimeUnit::Hour, 1)?;
            assert_eq!(new_time.to_string(), "01:00:00".to_string());
            Ok(())
        }

        #[test]
        fn time_remove_one_hour() -> Result<(), SpanError> {
            let time = Time::new(00, 00, 00)?;
            let new_time = time.update(TimeUnit::Hour, -1)?;
            assert_eq!(new_time.to_string(), "23:00:00".to_string());
            Ok(())
        }

        #[test]
        fn time_add_one_minute() -> Result<(), SpanError> {
            let time = Time::new(00, 00, 00)?;
            let new_time = time.update(TimeUnit::Minute, 1)?;
            assert_eq!(new_time.to_string(), "00:01:00".to_string());
            Ok(())
        }

        #[test]
        fn time_remove_one_minute() -> Result<(), SpanError> {
            let time = Time::new(00, 00, 00)?;
            let new_time = time.update(TimeUnit::Minute, -1)?;
            assert_eq!(new_time.to_string(), "23:59:00".to_string());
            Ok(())
        }

        #[test]
        fn time_add_one_second() -> Result<(), SpanError> {
            let time = Time::new(00, 00, 00)?;
            let new_time = time.update(TimeUnit::Second, 1)?;
            assert_eq!(new_time.to_string(), "00:00:01".to_string());
            Ok(())
        }

        #[test]
        fn time_remove_one_second() -> Result<(), SpanError> {
            let time = Time::new(00, 00, 00)?;
            let new_time = time.update(TimeUnit::Second, -1)?;
            assert_eq!(new_time.to_string(), "23:59:59".to_string());
            Ok(())
        }

        #[test]
        fn time_serialize() -> Result<(), SpanError> {
            let time = Time::new(12, 21, 46)?;
            let Ok(serialized) = serde_json::to_string(&time) else {
                panic!("Error while serializing time");
            };
            assert_eq!(
                serialized,
                "{\"time\":\"12:21:46\",\"format\":\"%H:%M:%S\"}".to_string()
            );
            Ok(())
        }

        #[test]
        fn time_deserialize() -> Result<(), SpanError> {
            let serialized = "{\"time\":\"12:21:46\",\"format\":\"%H:%M:%S\"}".to_string();
            let Ok(time) = serde_json::from_str::<Time>(&serialized) else {
                panic!("Error while deserializing time");
            };
            assert_eq!(time.to_string(), "12:21:46".to_string());
            assert_eq!(time.format, BASE_TIME_FORMAT.get().to_string());
            Ok(())
        }

        #[test]
        fn time_serialize_format() -> Result<(), SpanError> {
            let time = Time::new(12, 21, 46)?.format("T%H_%M_%S");
            let Ok(serialized) = serde_json::to_string(&time) else {
                panic!("Error while serializing time");
            };
            assert_eq!(
                serialized,
                "{\"time\":\"12:21:46\",\"format\":\"T%H_%M_%S\"}".to_string()
            );
            Ok(())
        }

        #[test]
        fn time_deserialize_format() -> Result<(), SpanError> {
            let serialized = "{\"time\":\"12:21:46\",\"format\":\"T%H_%M_%S\"}".to_string();
            let Ok(time) = serde_json::from_str::<Time>(&serialized) else {
                panic!("Error while deserializing time");
            };
            assert_eq!(time.to_string(), "T12_21_46".to_string());
            assert_eq!(time.format, "T%H_%M_%S".to_string());
            Ok(())
        }

        #[test]
        fn time_serialize_in_struct() -> Result<(), SpanError> {
            #[derive(Serialize)]
            struct Test {
                begin_at: Time,
            }
            let test = Test {
                begin_at: Time::new(23, 10, 09)?,
            };
            let Ok(serialized) = serde_json::to_string(&test) else {
                panic!("Error while serializing time");
            };
            assert_eq!(
                serialized,
                "{\"begin_at\":{\"time\":\"23:10:09\",\"format\":\"%H:%M:%S\"}}".to_string()
            );
            Ok(())
        }

        #[test]
        fn time_deserialize_in_struct() -> Result<(), SpanError> {
            #[derive(Deserialize)]
            struct Test {
                begin_at: Time,
            }
            let serialized =
                "{\"begin_at\":{\"time\":\"23:10:09\",\"format\":\"%H:%M:%S\"}}".to_string();
            let Ok(test) = serde_json::from_str::<Test>(&serialized) else {
                panic!("Error while deserializing time");
            };
            assert_eq!(test.begin_at.to_string(), "23:10:09".to_string());
            assert_eq!(test.begin_at.format, BASE_TIME_FORMAT.get().to_string());
            Ok(())
        }

        #[test]
        fn time_default_equal_midnight() -> Result<(), SpanError> {
            let time_built = Time::new(00, 00, 00)?;
            let midnight = Time::midnight();
            let default = Time::default();
            assert_eq!(time_built.to_string(), midnight.to_string());
            assert_eq!(time_built.format, midnight.format);
            assert_eq!(time_built.to_string(), default.to_string());
            assert_eq!(time_built.format, default.format);
            assert_eq!(midnight.to_string(), default.to_string());
            assert_eq!(midnight.format, default.format);
            Ok(())
        }

        #[test]
        fn next_second() -> Result<(), SpanError> {
            let mut time = Time::new(04, 23, 12)?;
            time = time.next(TimeUnit::Second)?;
            assert_eq!(time.to_string(), "04:23:13".to_string());
            Ok(())
        }

        #[test]
        fn next_minute() -> Result<(), SpanError> {
            let mut time = Time::new(11, 03, 22)?;
            time = time.next(TimeUnit::Minute)?;
            assert_eq!(time.to_string(), "11:04:22".to_string());
            Ok(())
        }

        #[test]
        fn next_hour_on_midnight() -> Result<(), SpanError> {
            let mut time = Time::new(23, 59, 34)?;
            time = time.next(TimeUnit::Hour)?;
            assert_eq!(time.to_string(), "00:59:34".to_string());
            Ok(())
        }

        #[test]
        fn matches_every_unit_in_time() -> Result<(), SpanError> {
            let time = Time::new(05, 23, 18)?;
            assert!(time.matches(TimeUnit::Hour, 5));
            assert!(time.matches(TimeUnit::Minute, 23));
            assert!(time.matches(TimeUnit::Second, 18));
            Ok(())
        }

        #[test]
        fn elapsed_three_minute() -> Result<(), SpanError> {
            let time = Time::new(00, 03, 00)?;
            let lhs = Time::new(00, 00, 00)?;
            assert_eq!(time.elapsed(&lhs), TimeDelta::try_minutes(3).unwrap());
            Ok(())
        }

        #[test]
        fn elapsed_seconds() -> Result<(), SpanError> {
            let time = Time::new(01, 21, 00)?;
            let lhs = Time::new(00, 00, 00)?;
            assert_eq!(time.elapsed(&lhs), TimeDelta::try_seconds(4860).unwrap());
            Ok(())
        }

        #[test]
        fn elapsed_multiple_units() -> Result<(), SpanError> {
            let time = Time::new(01, 01, 01)?;
            let lhs = Time::new(00, 00, 00)?;
            assert_eq!(
                time.elapsed(&lhs),
                TimeDelta::try_hours(1)
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
            let time = Time::new(01, 34, 45)?;
            let rhs = Time::new(00, 00, 00)?;
            let hours_in_between = time.unit_elapsed(&rhs, TimeUnit::Hour)?;
            let minutes_in_between = time.unit_elapsed(&rhs, TimeUnit::Minute)?;
            let seconds_in_between = time.unit_elapsed(&rhs, TimeUnit::Second)?;
            assert_eq!(hours_in_between, 1);
            assert_eq!(minutes_in_between, hours_in_between * 60 + 34);
            assert_eq!(seconds_in_between, minutes_in_between * 60 + 45);
            Ok(())
        }

        #[test]
        fn midnight() -> Result<(), SpanError> {
            let time = Time::midnight();
            assert_eq!(time.to_string(), "00:00:00".to_string());
            Ok(())
        }
    }
}

#[cfg(all(feature = "time", feature = "datetime"))]
mod datetime_into_time {
    use crate::GetInner;

    /// Convert a [DateTime] to a [Time]
    ///
    /// Only keep the time part of the [DateTime]
    ///
    /// # Example
    /// ```rust,ignore
    /// let datetime = DateTime::new(2021, 10, 10)?.with_time(12, 34, 56)?;
    /// let time = Time::from(datetime);
    /// assert_eq!(time.to_string(), "12:34:56".to_string());
    /// ```
    impl From<crate::datetime::DateTime> for crate::time::Time {
        fn from(value: crate::datetime::DateTime) -> Self {
            let time = value.datetime().time();
            Self {
                time,
                format: crate::time::BASE_TIME_FORMAT.get().to_string(),
            }
        }
    }

    #[cfg(test)]
    mod test {
        use crate::span::Span;

        #[test]
        fn datetime_into_time() -> Result<(), crate::error::SpanError> {
            let datetime = crate::datetime::DateTime::new(2021, 10, 10)?.with_time(12, 34, 56)?;
            let time = crate::time::Time::from(datetime);
            assert_eq!(time.to_string(), "12:34:56".to_string());
            Ok(())
        }

        #[test]
        #[ignore]
        fn datetime_into_time_wrong_format() -> Result<(), crate::error::SpanError> {
            let _span_builder = crate::builder::SpanBuilder::builder()
                .datetime_format("%Y-%m-%d %H:%M:%S")
                .time_format("%H_%M_%S")
                .build();
            let datetime = crate::datetime::DateTime::new(2021, 10, 10)?.with_time(12, 34, 56)?;
            let time = crate::time::Time::from(datetime);
            assert_eq!(time.to_string(), "12_34_56".to_string());
            Ok(())
        }
    }
}
