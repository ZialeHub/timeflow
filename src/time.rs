use std::ops::{Deref, DerefMut};

use chrono::{Local, NaiveDateTime, NaiveTime, TimeDelta, Timelike, Utc};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

use crate::{
    error::{ErrorContext, SpanError, TimeError},
    BASE_TIME_FORMAT,
};

/// [Serialize] the [NaiveTime] variable from [Time]
pub fn time_to_str<S: Serializer>(time: &NaiveTime, serializer: S) -> Result<S::Ok, S::Error> {
    time.format(BASE_TIME_FORMAT.get())
        .to_string()
        .serialize(serializer)
}

/// [Deserialize] the [NaiveTime] variable from [Time]
pub fn time_from_str<'de, D>(deserializer: D) -> Result<NaiveTime, D::Error>
where
    D: Deserializer<'de>,
{
    let time: String = Deserialize::deserialize(deserializer)?;
    NaiveTime::parse_from_str(&time, BASE_TIME_FORMAT.get()).map_err(de::Error::custom)
}

/// Unit to update [Time]
#[derive(Debug, Clone)]
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
    #[serde(serialize_with = "time_to_str", deserialize_with = "time_from_str")]
    pub time: NaiveTime,
    pub format: String,
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

impl Default for Time {
    fn default() -> Self {
        Self::midnight()
    }
}

impl Time {
    /// Create a new variable [Time] from the parameters `time` and `format`
    ///
    ///  See the [chrono::format::strftime] for the supported escape sequences of `format`.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let time = Time::new("09_27_00", "%H_%M_%S")?;
    ///
    /// assert_eq!(Time::build("11:35:19"), Time::new("11:35:19", BASE_TIME_FORMAT))
    /// ```
    ///
    /// # Errors
    ///
    /// Return an Err(_) if `time` is not formated with `format`
    pub fn new(time: impl ToString, format: impl ToString) -> Result<Self, SpanError> {
        let time = match NaiveTime::parse_from_str(&time.to_string(), &format.to_string()) {
            Ok(time) => time,
            Err(e) => return Err(SpanError::ParseFromStr(e)).err_ctx(TimeError),
        };
        Ok(Self {
            time,
            format: format.to_string(),
        })
    }

    /// Create a new variable [Time] from the parameter `time` formated with [BASE_TIME_FORMAT](static@BASE_TIME_FORMAT)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let time = Time::build("09:05:12")?;
    ///
    /// assert_eq!(Time::build("00:00:00"), Time::midnight())
    /// assert_eq!(Time::build("00:00:00"), Time::default())
    /// ```
    ///
    /// # Errors
    ///
    /// Return an Err(_) if the given `time` is not formated with [BASE_TIME_FORMAT](static@BASE_TIME_FORMAT)
    pub fn build(time: impl ToString) -> Result<Self, SpanError> {
        Self::new(time, BASE_TIME_FORMAT.get())
    }

    /// Getter for the time
    pub fn time(&self) -> NaiveTime {
        self.time
    }

    /// Setter for the format
    pub fn format(mut self, format: impl ToString) -> Self {
        self.format = format.to_string();
        self
    }

    /// Function to increase / decrease the time [Time] with [TimeUnit]
    pub fn update(&mut self, unit: TimeUnit, value: i32) -> Result<(), SpanError> {
        let delta_time = match unit {
            TimeUnit::Hour => TimeDelta::new(value as i64 * 60 * 60, 0),
            TimeUnit::Minute => TimeDelta::new(value as i64 * 60, 0),
            TimeUnit::Second => TimeDelta::new(value as i64, 0),
        };
        match delta_time {
            Some(delta_time) => {
                self.time += delta_time;
                Ok(())
            }
            None => Err(SpanError::InvalidUpdate(format!(
                "Cannot Add/Remove {} {:?} to/from {}",
                value, unit, self
            )))
            .err_ctx(TimeError),
        }
    }

    /// Go to the next [TimeUnit] from [Time]
    pub fn next(&mut self, unit: TimeUnit) -> Result<(), SpanError> {
        self.update(unit, 1)
    }

    /// Compare the [TimeUnit] from [Time] and value ([u32])
    pub fn matches(&self, unit: TimeUnit, value: u32) -> bool {
        match unit {
            TimeUnit::Hour => self.time.hour() == value,
            TimeUnit::Minute => self.time.minute() == value,
            TimeUnit::Second => self.time.second() == value,
        }
    }

    /// Return the current [Time] from the system
    pub fn now() -> Result<Self, SpanError> {
        Self::build(Local::now().format(BASE_TIME_FORMAT.get()))
    }

    /// Return midnight [Time]
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// assert_eq!(Time::build("00:00:00"), Time::midnight())
    /// assert_eq!(Time::build("00:00:00"), Time::default())
    /// ```
    pub fn midnight() -> Self {
        let time = NaiveTime::from_hms_opt(0, 0, 0).expect("Error Time midnight");
        Self {
            time,
            format: BASE_TIME_FORMAT.get().to_string(),
        }
    }

    /// Elapsed [TimeDelta] between two [Time]
    pub fn elapsed(&self, lhs: &Self) -> TimeDelta {
        self.time.signed_duration_since(lhs.time)
    }

    /// Number of [TimeUnit] between two [Time]
    pub fn unit_in_between(&self, unit: TimeUnit, lhs: &Self) -> i64 {
        match unit {
            TimeUnit::Hour => self.time.signed_duration_since(lhs.time).num_hours(),
            TimeUnit::Minute => self.time.signed_duration_since(lhs.time).num_minutes(),
            TimeUnit::Second => self.time.signed_duration_since(lhs.time).num_seconds(),
        }
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
        Self::new(time, format)
    }
}

impl TryFrom<(&str, &str)> for Time {
    type Error = SpanError;

    fn try_from((time, format): (&str, &str)) -> Result<Self, Self::Error> {
        Self::new(time, format)
    }
}

impl TryFrom<String> for Time {
    type Error = SpanError;

    fn try_from(time: String) -> Result<Self, Self::Error> {
        Self::build(time)
    }
}

impl TryFrom<&str> for Time {
    type Error = SpanError;

    fn try_from(time: &str) -> Result<Self, Self::Error> {
        Self::build(time)
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
        let mut time = Time::build("00:00:00")?;
        let new_time = time.update(TimeUnit::Hour, i32::MIN);
        assert_eq!(new_time, Ok(()));
        Ok(())
    }

    #[test]
    fn time_add_one_hour() -> Result<(), SpanError> {
        let mut time = Time::build("00:00:00")?;
        let new_time = time.update(TimeUnit::Hour, 1);
        assert_eq!(new_time, Ok(()));
        assert_eq!(time.to_string(), "01:00:00".to_string());
        Ok(())
    }

    #[test]
    fn time_remove_one_hour() -> Result<(), SpanError> {
        let mut time = Time::build("00:00:00")?;
        let new_time = time.update(TimeUnit::Hour, -1);
        assert_eq!(new_time, Ok(()));
        assert_eq!(time.to_string(), "23:00:00".to_string());
        Ok(())
    }

    #[test]
    fn test_time_add_one_minute() -> Result<(), SpanError> {
        let mut time = Time::build("00:00:00")?;
        let new_time = time.update(TimeUnit::Minute, 1);
        assert_eq!(new_time, Ok(()));
        assert_eq!(time.to_string(), "00:01:00".to_string());
        Ok(())
    }

    #[test]
    fn time_remove_one_minute() -> Result<(), SpanError> {
        let mut time = Time::build("00:00:00")?;
        let new_time = time.update(TimeUnit::Minute, -1);
        assert_eq!(new_time, Ok(()));
        assert_eq!(time.to_string(), "23:59:00".to_string());
        Ok(())
    }

    #[test]
    fn time_add_one_second() -> Result<(), SpanError> {
        let mut time = Time::build("00:00:00")?;
        let new_time = time.update(TimeUnit::Second, 1);
        assert_eq!(new_time, Ok(()));
        assert_eq!(time.to_string(), "00:00:01".to_string());
        Ok(())
    }

    #[test]
    fn time_remove_one_second() -> Result<(), SpanError> {
        let mut time = Time::build("00:00:00")?;
        let new_time = time.update(TimeUnit::Second, -1);
        assert_eq!(new_time, Ok(()));
        assert_eq!(time.to_string(), "23:59:59".to_string());
        Ok(())
    }

    #[test]
    fn time_serialize() -> Result<(), SpanError> {
        let time = Time::build("12:21:46")?;
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
        let time = Time::build("12:21:46")?.format("T%H_%M_%S");
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
    fn time_default_equal_midnight() -> Result<(), SpanError> {
        let time_built = Time::build("00:00:00")?;
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
        let mut time = Time::build("04:23:12")?;
        time.next(TimeUnit::Second)?;
        assert_eq!(time.to_string(), "04:23:13".to_string());
        Ok(())
    }

    #[test]
    fn next_minute() -> Result<(), SpanError> {
        let mut time = Time::build("11:03:22")?;
        time.next(TimeUnit::Minute)?;
        assert_eq!(time.to_string(), "11:04:22".to_string());
        Ok(())
    }

    #[test]
    fn next_hour_on_midnight() -> Result<(), SpanError> {
        let mut time = Time::build("23:59:34")?;
        time.next(TimeUnit::Hour)?;
        assert_eq!(time.to_string(), "00:59:34".to_string());
        Ok(())
    }

    #[test]
    fn matches_every_unit_in_time() -> Result<(), SpanError> {
        let time = Time::build("05:23:18")?;
        assert!(time.matches(TimeUnit::Hour, 5));
        assert!(time.matches(TimeUnit::Minute, 23));
        assert!(time.matches(TimeUnit::Second, 18));
        Ok(())
    }

    #[test]
    fn elapsed_three_minute() -> Result<(), SpanError> {
        let time = Time::build("00:03:00")?;
        let lhs = Time::build("00:00:00")?;
        assert_eq!(time.elapsed(&lhs), TimeDelta::try_minutes(3).unwrap());
        Ok(())
    }

    #[test]
    fn elapsed_seconds() -> Result<(), SpanError> {
        let time = Time::build("01:21:00")?;
        let lhs = Time::build("00:00:00")?;
        assert_eq!(time.elapsed(&lhs), TimeDelta::try_seconds(4860).unwrap());
        Ok(())
    }

    #[test]
    fn elapsed_multiple_units() -> Result<(), SpanError> {
        let time = Time::build("01:01:01")?;
        let lhs = Time::build("00:00:00")?;
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
    fn unit_in_between() -> Result<(), SpanError> {
        let time = Time::build("01:34:45")?;
        let lhs = Time::build("00:00:00")?;
        let hours_in_between = time.unit_in_between(TimeUnit::Hour, &lhs);
        let minutes_in_between = time.unit_in_between(TimeUnit::Minute, &lhs);
        let seconds_in_between = time.unit_in_between(TimeUnit::Second, &lhs);
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
