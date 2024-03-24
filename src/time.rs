use std::ops::{Deref, DerefMut};

use chrono::{Local, NaiveDateTime, NaiveTime, TimeDelta, Timelike};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

pub const BASE_TIME_FORMAT: &str = "%H:%M:%S";

/// [Serialize] the [NaiveTime] variable from [Time]
pub fn time_to_str<S: Serializer>(time: &NaiveTime, serializer: S) -> Result<S::Ok, S::Error> {
    time.format(BASE_TIME_FORMAT)
        .to_string()
        .serialize(serializer)
}

/// [Deserialize] the [NaiveTime] variable from [Time]
pub fn time_from_str<'de, D>(deserializer: D) -> Result<NaiveTime, D::Error>
where
    D: Deserializer<'de>,
{
    let time: String = Deserialize::deserialize(deserializer)?;
    NaiveTime::parse_from_str(&time, BASE_TIME_FORMAT).map_err(de::Error::custom)
}

/// Unit to update [Time]
#[derive(Debug, Clone)]
pub enum TimeUnit {
    Hour,
    Minute,
    Second,
}

// Structure to handle time management
///
/// Use [BASE_TIME_FORMAT] as default format for time
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    /// Create a new variable Time from the parameters `time` and `format`
    ///
    ///  See the [chrono::format::strftime] for the supported escape sequences of `format`.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// let time = Time::new("09_27_00", "%H_%M_%S")?;
    ///
    /// assert_eq!(Time::build("11:35:19"), Time::new("11:35:19", BASE_TIME_FORMAT))
    /// ```
    ///
    /// # Errors
    ///
    /// Return an Err(_) if `time` is not formated with `format`
    pub fn new(time: impl ToString, format: impl ToString) -> Result<Self, String> {
        let time = match NaiveTime::parse_from_str(&time.to_string(), &format.to_string()) {
            Ok(time) => time,
            Err(e) => return Err(format!("Error Time new: {e}")),
        };
        Ok(Self {
            time,
            format: format.to_string(),
        })
    }

    /// Create a new variable Time from the parameter `time`,
    /// if the time is formated with `BASE_TIME_FORMAT`
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// let time = Time::build("09:05:12")?;
    ///
    /// assert_eq!(Time::build("00:00:00"), Time::midnight())
    /// assert_eq!(Time::build("00:00:00"), Time::default())
    /// ```
    ///
    /// # Errors
    ///
    /// Return an Err(_) if the given `time` is not formated with `BASE_TIME_FORMAT`
    pub fn build(time: impl ToString) -> Result<Self, String> {
        Self::new(time, BASE_TIME_FORMAT)
    }

    /// Getter for the `time` [NaiveTime] variable
    pub fn time(&self) -> NaiveTime {
        self.time
    }

    /// Setter for the `format` [String] variable
    pub fn format(mut self, format: impl ToString) -> Self {
        self.format = format.to_string();
        self
    }

    /// Function to increase / decrease the time [Time] with [TimeUnit]
    pub fn update(&mut self, unit: TimeUnit, value: i32) -> Result<(), String> {
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
            None => Err(format!(
                "Cannot Add/Remove {} {:?} to/from {}",
                value, unit, self
            )),
        }
    }

    /// Go to the next [TimeUnit] from [Time]
    pub fn next(&mut self, unit: TimeUnit) -> Result<(), String> {
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

    /// Return the current [Time]
    pub fn now() -> Result<Self, String> {
        Self::build(Local::now().format(BASE_TIME_FORMAT))
    }

    /// Return midnight [Time]
    ///
    /// # Example
    ///
    /// ```rust,no_run
    ///
    /// assert_eq!(Time::build("00:00:00"), Time::midnight())
    /// assert_eq!(Time::build("00:00:00"), Time::default())
    /// ```
    pub fn midnight() -> Self {
        let time = NaiveTime::from_hms_opt(0, 0, 0).expect("Error Time midnight");
        Self {
            time,
            format: BASE_TIME_FORMAT.to_string(),
        }
    }

    /// Return a [bool] to know if the [Time] is in the future
    pub fn is_in_future(&self) -> Result<bool, String> {
        Ok(self.time > Self::now()?.time)
    }

    /// Elapsed [TimeDelta] between two [Time]
    pub fn elapsed(&self, lhs: &Self) -> TimeDelta {
        self.time.signed_duration_since(lhs.time)
    }

    /// Number of [TimeUnit] between two [Time]
    pub fn unit_in_between(&self, unit: TimeUnit, lhs: &Self) -> i64 {
        match unit {
            TimeUnit::Hour => self.time.hour() as i64 - lhs.time.hour() as i64,
            TimeUnit::Minute => self.time.minute() as i64 - lhs.time.minute() as i64,
            TimeUnit::Second => self.time.second() as i64 - lhs.time.second() as i64,
        }
    }
}

impl From<NaiveDateTime> for Time {
    fn from(datetime: NaiveDateTime) -> Self {
        Self {
            time: datetime.time(),
            format: BASE_TIME_FORMAT.to_string(),
        }
    }
}

impl From<NaiveTime> for Time {
    fn from(time: NaiveTime) -> Self {
        Self {
            time,
            format: BASE_TIME_FORMAT.to_string(),
        }
    }
}

impl TryFrom<(String, String)> for Time {
    type Error = String;

    fn try_from((time, format): (String, String)) -> Result<Self, Self::Error> {
        Self::new(time, format)
    }
}

impl TryFrom<(&str, &str)> for Time {
    type Error = String;

    fn try_from((time, format): (&str, &str)) -> Result<Self, Self::Error> {
        Self::new(time, format)
    }
}

impl TryFrom<String> for Time {
    type Error = String;

    fn try_from(time: String) -> Result<Self, Self::Error> {
        Self::build(time)
    }
}

impl TryFrom<&str> for Time {
    type Error = String;

    fn try_from(time: &str) -> Result<Self, Self::Error> {
        Self::build(time)
    }
}

#[cfg(test)]
pub mod test {
    use super::*;

    #[test]
    fn test_time_add_overflow() -> Result<(), String> {
        let mut time = Time::build("00:00:00")?;
        let new_time = time.update(TimeUnit::Hour, i32::MIN);
        assert_eq!(new_time, Ok(()));
        Ok(())
    }

    #[test]
    fn test_time_add_one_hour() -> Result<(), String> {
        let mut time = Time::build("00:00:00")?;
        let new_time = time.update(TimeUnit::Hour, 1);
        assert_eq!(new_time, Ok(()));
        assert_eq!(time.to_string(), "01:00:00".to_string());
        Ok(())
    }

    #[test]
    fn test_time_remove_one_hour() -> Result<(), String> {
        let mut time = Time::build("00:00:00")?;
        let new_time = time.update(TimeUnit::Hour, -1);
        assert_eq!(new_time, Ok(()));
        assert_eq!(time.to_string(), "23:00:00".to_string());
        Ok(())
    }

    #[test]
    fn test_time_add_one_minute() -> Result<(), String> {
        let mut time = Time::build("00:00:00")?;
        let new_time = time.update(TimeUnit::Minute, 1);
        assert_eq!(new_time, Ok(()));
        assert_eq!(time.to_string(), "00:01:00".to_string());
        Ok(())
    }

    #[test]
    fn test_time_remove_one_minute() -> Result<(), String> {
        let mut time = Time::build("00:00:00")?;
        let new_time = time.update(TimeUnit::Minute, -1);
        assert_eq!(new_time, Ok(()));
        assert_eq!(time.to_string(), "23:59:00".to_string());
        Ok(())
    }

    #[test]
    fn test_time_add_one_second() -> Result<(), String> {
        let mut time = Time::build("00:00:00")?;
        let new_time = time.update(TimeUnit::Second, 1);
        assert_eq!(new_time, Ok(()));
        assert_eq!(time.to_string(), "00:00:01".to_string());
        Ok(())
    }

    #[test]
    fn test_time_remove_one_second() -> Result<(), String> {
        let mut time = Time::build("00:00:00")?;
        let new_time = time.update(TimeUnit::Second, -1);
        assert_eq!(new_time, Ok(()));
        assert_eq!(time.to_string(), "23:59:59".to_string());
        Ok(())
    }

    #[test]
    fn test_time_serialize() -> Result<(), String> {
        let time = Time::build("12:21:46")?;
        let Ok(serialized) = serde_json::to_string(&time) else {
            return Err("Error while serializing time".to_string());
        };
        assert_eq!(
            serialized,
            "{\"time\":\"12:21:46\",\"format\":\"%H:%M:%S\"}".to_string()
        );
        Ok(())
    }

    #[test]
    fn test_time_deserialize() -> Result<(), String> {
        let serialized = "{\"time\":\"12:21:46\",\"format\":\"%H:%M:%S\"}".to_string();
        let Ok(time) = serde_json::from_str::<Time>(&serialized) else {
            return Err("Error while deserializing time".to_string());
        };
        assert_eq!(time.to_string(), "12:21:46".to_string());
        assert_eq!(time.format, BASE_TIME_FORMAT.to_string());
        Ok(())
    }

    #[test]
    fn test_time_serialize_format() -> Result<(), String> {
        let time = Time::build("12:21:46")?.format("T%H_%M_%S");
        let Ok(serialized) = serde_json::to_string(&time) else {
            return Err("Error while serializing time".to_string());
        };
        assert_eq!(
            serialized,
            "{\"time\":\"12:21:46\",\"format\":\"T%H_%M_%S\"}".to_string()
        );
        Ok(())
    }

    #[test]
    fn test_time_deserialize_format() -> Result<(), String> {
        let serialized = "{\"time\":\"12:21:46\",\"format\":\"T%H_%M_%S\"}".to_string();
        let Ok(time) = serde_json::from_str::<Time>(&serialized) else {
            return Err("Error while deserializing time".to_string());
        };
        assert_eq!(time.to_string(), "T12_21_46".to_string());
        assert_eq!(time.format, "T%H_%M_%S".to_string());
        Ok(())
    }

    #[test]
    fn test_time_default_equal_midnight() -> Result<(), String> {
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
}
