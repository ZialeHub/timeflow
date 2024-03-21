use std::ops::{Deref, DerefMut};

use chrono::{Local, NaiveDateTime, NaiveTime, TimeDelta, Timelike};

const BASE_TIME_FORMAT: &str = "%H:%M:%S";

#[derive(Debug, Clone)]
enum TimeUnit {
    Hour,
    Minute,
    Second,
}

/// Time structure to handle time management
///
/// const BASE_TIME_FORMAT: &str = "%H:%M:%S";
///
/// BASE_TIME_FORMAT is the default format for time
// TODO Find a way to implement Serialize and Deserialize for Time
#[derive(Debug, Clone)]
struct Time {
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

impl Time {
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

    pub fn build(time: impl ToString) -> Result<Self, String> {
        Self::new(time, BASE_TIME_FORMAT)
    }

    pub fn time(&self) -> NaiveTime {
        self.time
    }

    pub fn format(mut self, format: impl ToString) -> Self {
        self.format = format.to_string();
        self
    }

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

    pub fn next(&mut self, unit: TimeUnit) -> Result<(), String> {
        self.update(unit, 1)
    }

    pub fn matches(&self, unit: TimeUnit, value: u32) -> Result<bool, String> {
        match unit {
            TimeUnit::Hour => Ok(self.time.hour() == value),
            TimeUnit::Minute => Ok(self.time.minute() == value),
            TimeUnit::Second => Ok(self.time.second() == value),
        }
    }

    pub fn now() -> Result<Self, String> {
        Self::build(Local::now().format(BASE_TIME_FORMAT))
    }

    pub fn midnight() -> Result<Self, String> {
        Self::build("00:00:00")
    }

    pub fn is_in_future(&self) -> Result<bool, String> {
        Ok(self.time > Self::now()?.time)
    }

    pub fn elapsed(&self, lhs: &Self) -> TimeDelta {
        self.time.signed_duration_since(lhs.time)
    }

    pub fn unit_in_between(&self, unit: TimeUnit, lhs: &Self) -> i64 {
        match unit {
            TimeUnit::Hour => self.time.hour() as i64 - lhs.time.hour() as i64,
            TimeUnit::Minute => self.time.minute() as i64 - lhs.time.minute() as i64,
            TimeUnit::Second => self.time.second() as i64 - lhs.time.second() as i64,
        }
    }
}

impl From<NaiveDateTime> for Time {
    fn from(date: NaiveDateTime) -> Self {
        Self {
            time: date.time(),
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
        let mut datetime = Time::build("00:00:00")?;
        let new_datetime = datetime.update(TimeUnit::Hour, i32::MIN);
        assert_eq!(new_datetime, Ok(()));
        Ok(())
    }

    #[test]
    fn test_time_add_one_hour() -> Result<(), String> {
        let mut datetime = Time::build("00:00:00")?;
        let new_datetime = datetime.update(TimeUnit::Hour, 1);
        assert_eq!(new_datetime, Ok(()));
        assert_eq!(datetime.to_string(), "01:00:00".to_string());
        Ok(())
    }

    #[test]
    fn test_time_remove_one_hour() -> Result<(), String> {
        let mut datetime = Time::build("00:00:00")?;
        let new_datetime = datetime.update(TimeUnit::Hour, -1);
        assert_eq!(new_datetime, Ok(()));
        assert_eq!(datetime.to_string(), "23:00:00".to_string());
        Ok(())
    }

    #[test]
    fn test_time_add_one_minute() -> Result<(), String> {
        let mut datetime = Time::build("00:00:00")?;
        let new_datetime = datetime.update(TimeUnit::Minute, 1);
        assert_eq!(new_datetime, Ok(()));
        assert_eq!(datetime.to_string(), "00:01:00".to_string());
        Ok(())
    }

    #[test]
    fn test_time_remove_one_minute() -> Result<(), String> {
        let mut datetime = Time::build("00:00:00")?;
        let new_datetime = datetime.update(TimeUnit::Minute, -1);
        assert_eq!(new_datetime, Ok(()));
        assert_eq!(datetime.to_string(), "23:59:00".to_string());
        Ok(())
    }

    #[test]
    fn test_time_add_one_second() -> Result<(), String> {
        let mut datetime = Time::build("00:00:00")?;
        let new_datetime = datetime.update(TimeUnit::Second, 1);
        assert_eq!(new_datetime, Ok(()));
        assert_eq!(datetime.to_string(), "00:00:01".to_string());
        Ok(())
    }

    #[test]
    fn test_time_remove_one_second() -> Result<(), String> {
        let mut datetime = Time::build("00:00:00")?;
        let new_datetime = datetime.update(TimeUnit::Second, -1);
        assert_eq!(new_datetime, Ok(()));
        assert_eq!(datetime.to_string(), "23:59:59".to_string());
        Ok(())
    }
}
