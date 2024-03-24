use std::ops::{Deref, DerefMut};

use chrono::{Datelike, Days, Duration, Local, Months, NaiveDate, NaiveDateTime};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

use crate::{TimeRSError, TimeResult, Type};

const BASE_DATE_FORMAT: &str = "%Y-%m-%d";

pub fn date_to_str<S: Serializer>(date: &NaiveDate, serializer: S) -> Result<S::Ok, S::Error> {
    date.format(BASE_DATE_FORMAT)
        .to_string()
        .serialize(serializer)
}

pub fn date_from_str<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
where
    D: Deserializer<'de>,
{
    let date: String = Deserialize::deserialize(deserializer)?;
    NaiveDate::parse_from_str(&date, BASE_DATE_FORMAT).map_err(de::Error::custom)
}

#[derive(Debug, Clone)]
enum DateUnit {
    Year,
    Month,
    Day,
}

/// Date structure to handle date management
///
/// const BASE_DATE_FORMAT: &str = "%Y-%m-%d";
///
/// BASE_DATE_FORMAT is the default format for date
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Date {
    #[serde(serialize_with = "date_to_str", deserialize_with = "date_from_str")]
    pub date: NaiveDate,
    pub format: String,
}

impl std::fmt::Display for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.date.format(&self.format))
    }
}

impl Deref for Date {
    type Target = NaiveDate;

    fn deref(&self) -> &Self::Target {
        &self.date
    }
}

impl DerefMut for Date {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.date
    }
}

impl Date {
    pub fn new(date: impl ToString, format: impl ToString) -> TimeResult<Self> {
        let date = match NaiveDate::parse_from_str(&date.to_string(), &format.to_string()) {
            Ok(date) => date,
            Err(e) => return Err(TimeRSError::ParseFromStr(Type::Date, e.to_string())),
        };
        Ok(Self {
            date,
            format: format.to_string(),
        })
    }

    pub fn build(date: impl ToString) -> TimeResult<Self> {
        Self::new(date, BASE_DATE_FORMAT)
    }

    pub fn date(&self) -> NaiveDate {
        self.date
    }

    pub fn format(mut self, format: impl ToString) -> Self {
        self.format = format.to_string();
        self
    }

    pub fn update(&mut self, unit: DateUnit, value: i32) -> TimeResult<()> {
        let date = match unit {
            DateUnit::Year if value > 0 => {
                self.date.checked_add_months(Months::new(value as u32 * 12))
            }
            DateUnit::Year => self
                .date
                .checked_sub_months(Months::new(value.unsigned_abs() * 12)),
            DateUnit::Month if value > 0 => self.date.checked_add_months(Months::new(value as u32)),
            DateUnit::Month => self
                .date
                .checked_sub_months(Months::new(value.unsigned_abs())),
            DateUnit::Day if value > 0 => self.date.checked_add_days(Days::new(value as u64)),
            DateUnit::Day => self
                .date
                .checked_sub_days(Days::new(value.unsigned_abs() as u64)),
        };
        match date {
            Some(date) => {
                self.date = date;
                Ok(())
            }
            None => Err(TimeRSError::InvalidUpdate(
                Type::Date,
                format!("Cannot Add/Remove {} {:?} to/from {}", value, unit, self),
            )),
        }
    }

    pub fn next(&mut self, unit: DateUnit) -> TimeResult<()> {
        self.update(unit, 1)
    }

    pub fn matches(&self, unit: DateUnit, value: i32) -> bool {
        match unit {
            DateUnit::Year => self.date.year() == value,
            DateUnit::Month => self.date.month() == value as u32,
            DateUnit::Day => self.date.day() == value as u32,
        }
    }

    pub fn today() -> TimeResult<Self> {
        Self::build(Local::now().format(BASE_DATE_FORMAT))
    }

    pub fn is_in_future(&self) -> TimeResult<bool> {
        Ok(self.date > Self::today()?.date)
    }

    pub fn elapsed(&self, lhs: &Self) -> Duration {
        self.date.signed_duration_since(lhs.date)
    }

    pub fn unit_in_between(&self, unit: DateUnit, lhs: &Self) -> i64 {
        match unit {
            DateUnit::Year => self.date.year() as i64 - lhs.date.year() as i64,
            DateUnit::Month => self.date.month() as i64 - lhs.date.month() as i64,
            DateUnit::Day => self.date.day() as i64 - lhs.date.day() as i64,
        }
    }
}

impl From<NaiveDateTime> for Date {
    fn from(value: NaiveDateTime) -> Self {
        Self {
            date: value.date(),
            format: BASE_DATE_FORMAT.to_string(),
        }
    }
}

impl From<NaiveDate> for Date {
    fn from(value: NaiveDate) -> Self {
        Self {
            date: value,
            format: BASE_DATE_FORMAT.to_string(),
        }
    }
}

impl TryFrom<(String, String)> for Date {
    type Error = TimeRSError;
    fn try_from((date, format): (String, String)) -> Result<Self, Self::Error> {
        Self::new(date, format)
    }
}

impl TryFrom<(&str, &str)> for Date {
    type Error = TimeRSError;
    fn try_from((date, format): (&str, &str)) -> Result<Self, Self::Error> {
        Self::new(date, format)
    }
}

impl TryFrom<String> for Date {
    type Error = TimeRSError;
    fn try_from(date: String) -> Result<Self, Self::Error> {
        Self::new(date, BASE_DATE_FORMAT)
    }
}

impl TryFrom<&str> for Date {
    type Error = TimeRSError;
    fn try_from(date: &str) -> Result<Self, Self::Error> {
        Self::new(date, BASE_DATE_FORMAT)
    }
}

#[cfg(test)]
pub mod test {
    use super::*;

    #[test]
    fn test_date_add_overflow() -> TimeResult<()> {
        let mut date = Date::build("2023-10-09")?;
        let new_date = date.update(DateUnit::Day, i32::MAX);
        assert_eq!(
            new_date,
            Err(TimeRSError::InvalidUpdate(
                Type::Date,
                "Cannot Add/Remove 2147483647 Day to/from 2023-10-09".to_string()
            ))
        );
        Ok(())
    }

    #[test]
    fn test_date_add_one_year() -> TimeResult<()> {
        let mut date = Date::build("2023-10-09")?;
        let new_date = date.update(DateUnit::Year, 1);
        assert_eq!(new_date, Ok(()));
        assert_eq!(date.to_string(), "2024-10-09".to_string());
        Ok(())
    }

    #[test]
    fn test_date_remove_one_year() -> TimeResult<()> {
        let mut date = Date::build("2023-10-09")?;
        let new_date = date.update(DateUnit::Year, -1);
        assert_eq!(new_date, Ok(()));
        assert_eq!(date.to_string(), "2022-10-09".to_string());
        Ok(())
    }

    #[test]
    fn test_date_add_one_month() -> TimeResult<()> {
        let mut date = Date::build("2023-10-09")?;
        let new_date = date.update(DateUnit::Month, 1);
        assert_eq!(new_date, Ok(()));
        assert_eq!(date.to_string(), "2023-11-09".to_string());
        Ok(())
    }

    #[test]
    fn test_date_remove_one_month() -> TimeResult<()> {
        let mut date = Date::build("2023-10-09")?;
        let new_date = date.update(DateUnit::Month, -1);
        assert_eq!(new_date, Ok(()));
        assert_eq!(date.to_string(), "2023-09-09".to_string());
        Ok(())
    }

    #[test]
    fn test_date_add_one_day() -> TimeResult<()> {
        let mut date = Date::build("2023-10-09")?;
        let new_date = date.update(DateUnit::Day, 1);
        assert_eq!(new_date, Ok(()));
        assert_eq!(date.to_string(), "2023-10-10".to_string());
        Ok(())
    }

    #[test]
    fn test_date_remove_one_day() -> TimeResult<()> {
        let mut date = Date::build("2023-10-09")?;
        let new_date = date.update(DateUnit::Day, -1);
        assert_eq!(new_date, Ok(()));
        assert_eq!(date.to_string(), "2023-10-08".to_string());
        Ok(())
    }

    #[test]
    fn test_date_serialize() -> TimeResult<()> {
        let date = Date::build("2023-10-09")?;
        let Ok(serialized) = serde_json::to_string(&date) else {
            panic!("Error while serializing date");
        };
        assert_eq!(
            serialized,
            "{\"date\":\"2023-10-09\",\"format\":\"%Y-%m-%d\"}".to_string()
        );
        Ok(())
    }

    #[test]
    fn test_date_deserialize() -> TimeResult<()> {
        let serialized = "{\"date\":\"2023-10-09\",\"format\":\"%Y-%m-%d\"}".to_string();
        let Ok(date) = serde_json::from_str::<Date>(&serialized) else {
            panic!("Error while deserializing date");
        };
        assert_eq!(date.to_string(), "2023-10-09".to_string());
        assert_eq!(date.format, BASE_DATE_FORMAT.to_string());
        Ok(())
    }

    #[test]
    fn test_date_serialize_format() -> TimeResult<()> {
        let date = Date::build("2023-10-09")?.format("%d/%m/%Y");
        let Ok(serialized) = serde_json::to_string(&date) else {
            panic!("Error while serializing date");
        };
        assert_eq!(
            serialized,
            "{\"date\":\"2023-10-09\",\"format\":\"%d/%m/%Y\"}".to_string()
        );
        Ok(())
    }

    #[test]
    fn test_date_deserialize_format() -> TimeResult<()> {
        let serialized = "{\"date\":\"2023-10-09\",\"format\":\"%d/%m/%Y\"}".to_string();
        let Ok(date) = serde_json::from_str::<Date>(&serialized) else {
            panic!("Error while deserializing date");
        };
        assert_eq!(date.to_string(), "09/10/2023".to_string());
        assert_eq!(date.format, "%d/%m/%Y".to_string());
        Ok(())
    }
}
