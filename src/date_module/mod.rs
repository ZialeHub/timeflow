#[cfg(feature = "date")]
pub mod date {
    use std::{
        ops::{Deref, DerefMut},
        sync::{LazyLock, RwLock},
    };

    use chrono::{Datelike, Days, Duration, Local, Months, NaiveDate, NaiveDateTime, Utc};
    use serde::{Deserialize, Serialize};

    use crate::{
        error::{DateError, ErrorContext, SpanError},
        BaseFormat, GetInner,
    };

    pub(crate) static BASE_DATE_FORMAT: BaseFormat<&'static str> =
        LazyLock::new(|| RwLock::new("%Y-%m-%d"));

    /// Unit to update [Date]
    #[derive(Debug, Clone)]
    pub enum DateUnit {
        Year,
        Month,
        Day,
    }

    /// Structure to handle date management
    ///
    /// Use [BASE_DATE_FORMAT](static@BASE_DATE_FORMAT) as default format for date
    #[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Clone, Serialize, Deserialize)]
    pub struct Date {
        pub(crate) date: NaiveDate,
        pub(crate) format: String,
    }

    impl Default for Date {
        fn default() -> Self {
            Self {
                date: NaiveDate::default(),
                format: BASE_DATE_FORMAT.get().to_string(),
            }
        }
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
        /// Create a new variable [Date] from the parameters `date` and `format`
        ///
        ///  See the [chrono::format::strftime] for the supported escape sequences of `format`.
        ///
        /// # Example
        ///
        /// ```rust,ignore
        /// let date = Date::new("09/27/2024", "%m/%d/%Y")?;
        /// ```
        ///
        /// # Errors
        ///
        /// Return an Err(_) if `time` is not formated with `format`
        pub fn new(date: impl ToString, format: impl ToString) -> Result<Self, SpanError> {
            let date = match NaiveDate::parse_from_str(&date.to_string(), &format.to_string()) {
                Ok(date) => date,
                Err(e) => return Err(SpanError::ParseFromStr(e)).err_ctx(DateError),
            };
            Ok(Self {
                date,
                format: format.to_string(),
            })
        }

        /// Create a new variable [Date] from the parameter `date` formated with [BASE_DATE_FORMAT](static@BASE_DATE_FORMAT)
        ///
        /// # Example
        ///
        /// ```rust,ignore
        /// let date = Date::build("2024-09-27")?;
        /// ```
        ///
        /// # Errors
        ///
        /// Return an Err(_) if the given `date` is not formated with [BASE_DATE_FORMAT](static@BASE_DATE_FORMAT)
        pub fn build(date: impl ToString) -> Result<Self, SpanError> {
            Self::new(date, BASE_DATE_FORMAT.get())
        }

        /// Getter for the date
        pub fn date(&self) -> NaiveDate {
            self.date
        }

        /// Setter for the format
        ///
        ///  See the [chrono::format::strftime] for the supported escape sequences of `format`.
        pub fn format(mut self, format: impl ToString) -> Self {
            self.format = format.to_string();
            self
        }

        /// Function to increase / decrease the date [Date] by [DateUnit]
        ///
        /// # Example
        /// ```rust,ignore
        /// let mut date = Date::build("2023-10-09")?;
        /// date.update(DateUnit::Year, 1)?;
        /// assert_eq!(date.to_string(), "2024-10-09".to_string());
        ///
        /// let mut date = Date::build("2023-10-09")?;
        /// date.update(DateUnit::Year, -1)?;
        /// assert_eq!(date.to_string(), "2022-10-09".to_string());
        /// ```
        ///
        /// # Errors
        /// Return an Err(_) if the operation is not possible or if [chrono] fails
        pub fn update(&self, unit: DateUnit, value: i32) -> Result<Self, SpanError> {
            let date = match unit {
                DateUnit::Year if value > 0 => {
                    self.date.checked_add_months(Months::new(value as u32 * 12))
                }
                DateUnit::Year => self
                    .date
                    .checked_sub_months(Months::new(value.unsigned_abs() * 12)),
                DateUnit::Month if value > 0 => {
                    self.date.checked_add_months(Months::new(value as u32))
                }
                DateUnit::Month => self
                    .date
                    .checked_sub_months(Months::new(value.unsigned_abs())),
                DateUnit::Day if value > 0 => self.date.checked_add_days(Days::new(value as u64)),
                DateUnit::Day => self
                    .date
                    .checked_sub_days(Days::new(value.unsigned_abs() as u64)),
            };
            match date {
                Some(date) => Ok(Self {
                    date,
                    format: self.format.clone(),
                }),
                None => Err(SpanError::InvalidUpdate(format!(
                    "Cannot Add/Remove {} {:?} to/from {}",
                    value, unit, self
                )))
                .err_ctx(DateError),
            }
        }

        /// Go to the next [DateUnit] from [Date]
        ///
        /// # Example
        /// ```rust,ignore
        /// let mut date = Date::build("2023-01-31")?;
        /// date.next(DateUnit::Month)?;
        /// assert_eq!(date.to_string(), "2023-02-28".to_string());
        ///
        /// let mut date = Date::build("2023-10-09")?;
        /// date.next(DateUnit::Day)?;
        /// assert_eq!(date.to_string(), "2023-10-10".to_string());
        /// ```
        pub fn next(&self, unit: DateUnit) -> Result<Self, SpanError> {
            self.update(unit, 1)
        }

        /// Compare the [DateUnit] from [Date] and value ([i32])
        ///
        /// # Example
        /// ```rust,ignore
        /// let date = Date::build("2023-10-09")?;
        /// assert!(date.matches(DateUnit::Year, 2023));
        /// assert!(date.matches(DateUnit::Month, 10));
        /// assert!(date.matches(DateUnit::Day, 9));
        /// ```
        pub fn matches(&self, unit: DateUnit, value: i32) -> bool {
            match unit {
                DateUnit::Year => self.date.year() == value,
                DateUnit::Month => self.date.month() == value as u32,
                DateUnit::Day => self.date.day() == value as u32,
            }
        }

        /// Return the current [Date] from the system
        pub fn today() -> Result<Self, SpanError> {
            Self::build(Local::now().format(BASE_DATE_FORMAT.get()))
        }

        /// Return a [bool] to know if the [Date] is in the future
        ///
        /// # Example
        /// ```rust,ignore
        /// // If today is 2024-01-01
        /// let date = Date::build("2023-10-09")?;
        /// assert!(!date.is_in_future()?);
        ///
        /// // If today is 2022-01-01
        /// let date = Date::build("2023-10-09")?;
        /// assert!(date.is_in_future()?);
        /// ```
        pub fn is_in_future(&self) -> Result<bool, SpanError> {
            Ok(self.date > Self::today()?.date)
        }

        /// Elapsed [Duration] between two [Date]
        ///
        /// # Example
        /// ```rust,ignore
        /// let rhs = Date::build("2023-10-20")?;
        /// let lhs = Date::build("2023-10-09")?;
        /// assert_eq!(rhs.elapsed(&lhs), TimeDelta::try_days(11).unwrap());
        /// ```
        pub fn elapsed(&self, lhs: &Self) -> Duration {
            self.date.signed_duration_since(lhs.date)
        }

        /// Number of [DateUnit] between two [Date]
        ///
        /// # Example
        /// ```rust,ignore
        /// let lhs = Date::build("2023-10-20")?;
        /// let rhs = Date::build("2023-10-09")?;
        /// assert_eq!(lhs.unit_elapsed(&rhs, DateUnit::Day), Ok(11));
        /// ```
        pub fn unit_elapsed(&self, rhs: &Self, unit: DateUnit) -> Result<i64, SpanError> {
            Ok(match unit {
                DateUnit::Year => self.date.year() as i64 - rhs.date.year() as i64,
                DateUnit::Month => {
                    self.date.year() as i64 * 12 + self.date.month() as i64
                        - (rhs.date.year() as i64 * 12 + rhs.date.month() as i64)
                }
                DateUnit::Day => {
                    let self_utc: chrono::DateTime<Utc> = self.try_into()?;
                    let lhs_utc: chrono::DateTime<Utc> = rhs.try_into()?;
                    self_utc.signed_duration_since(lhs_utc).num_days()
                }
            })
        }
    }

    impl From<NaiveDateTime> for Date {
        fn from(value: NaiveDateTime) -> Self {
            Self {
                date: value.date(),
                format: BASE_DATE_FORMAT.get().to_string(),
            }
        }
    }

    impl From<NaiveDate> for Date {
        fn from(value: NaiveDate) -> Self {
            Self {
                date: value,
                format: BASE_DATE_FORMAT.get().to_string(),
            }
        }
    }

    impl TryFrom<(String, String)> for Date {
        type Error = SpanError;
        fn try_from((date, format): (String, String)) -> Result<Self, Self::Error> {
            Self::new(date, format)
        }
    }

    impl TryFrom<(&str, &str)> for Date {
        type Error = SpanError;
        fn try_from((date, format): (&str, &str)) -> Result<Self, Self::Error> {
            Self::new(date, format)
        }
    }

    impl TryFrom<String> for Date {
        type Error = SpanError;
        fn try_from(date: String) -> Result<Self, Self::Error> {
            Self::new(date, BASE_DATE_FORMAT.get())
        }
    }

    impl TryFrom<&str> for Date {
        type Error = SpanError;
        fn try_from(date: &str) -> Result<Self, Self::Error> {
            Self::new(date, BASE_DATE_FORMAT.get())
        }
    }

    impl TryFrom<chrono::DateTime<Utc>> for Date {
        type Error = SpanError;
        fn try_from(value: chrono::DateTime<Utc>) -> Result<Self, Self::Error> {
            Ok(value.naive_utc().into())
        }
    }

    impl TryFrom<&Date> for chrono::DateTime<Utc> {
        type Error = SpanError;
        fn try_from(value: &Date) -> Result<Self, Self::Error> {
            let date = value.date;
            match Utc::now()
                .with_year(date.year())
                .and_then(|utc| utc.with_day(date.day()))
                .and_then(|utc| utc.with_month(date.month()))
            {
                Some(utc) => Ok(utc),
                None => Err(SpanError::InvalidUtc).err_ctx(DateError),
            }
        }
    }

    #[cfg(test)]
    pub mod test {
        use chrono::TimeDelta;

        use super::*;

        #[test]
        fn date_add_overflow() -> Result<(), SpanError> {
            let date = Date::build("2023-10-09")?;
            let new_date = date.update(DateUnit::Day, i32::MAX);
            assert_eq!(
                new_date,
                Err(SpanError::InvalidUpdate(
                    "Cannot Add/Remove 2147483647 Day to/from 2023-10-09".to_string()
                ))
                .err_ctx(DateError)
            );
            Ok(())
        }

        #[test]
        fn date_add_one_year() -> Result<(), SpanError> {
            let date = Date::build("2023-10-09")?;
            let new_date = date.update(DateUnit::Year, 1)?;
            assert_eq!(new_date.to_string(), "2024-10-09".to_string());
            Ok(())
        }

        #[test]
        fn date_remove_one_year() -> Result<(), SpanError> {
            let date = Date::build("2023-10-09")?;
            let new_date = date.update(DateUnit::Year, -1)?;
            assert_eq!(new_date.to_string(), "2022-10-09".to_string());
            Ok(())
        }

        #[test]
        fn date_add_one_month() -> Result<(), SpanError> {
            let date = Date::build("2023-10-09")?;
            let new_date = date.update(DateUnit::Month, 1)?;
            assert_eq!(new_date.to_string(), "2023-11-09".to_string());
            Ok(())
        }

        #[test]
        fn date_remove_one_month() -> Result<(), SpanError> {
            let date = Date::build("2023-10-09")?;
            let new_date = date.update(DateUnit::Month, -1)?;
            assert_eq!(new_date.to_string(), "2023-09-09".to_string());
            Ok(())
        }

        #[test]
        fn date_add_one_day() -> Result<(), SpanError> {
            let date = Date::build("2023-10-09")?;
            let new_date = date.update(DateUnit::Day, 1)?;
            assert_eq!(new_date.to_string(), "2023-10-10".to_string());
            Ok(())
        }

        #[test]
        fn date_remove_one_day() -> Result<(), SpanError> {
            let date = Date::build("2023-10-09")?;
            let new_date = date.update(DateUnit::Day, -1)?;
            assert_eq!(new_date.to_string(), "2023-10-08".to_string());
            Ok(())
        }

        #[test]
        fn date_serialize() -> Result<(), SpanError> {
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
        fn date_deserialize() -> Result<(), SpanError> {
            let serialized = "{\"date\":\"2023-10-09\",\"format\":\"%Y-%m-%d\"}".to_string();
            let Ok(date) = serde_json::from_str::<Date>(&serialized) else {
                panic!("Error while deserializing date");
            };
            assert_eq!(date.to_string(), "2023-10-09".to_string());
            assert_eq!(date.format, BASE_DATE_FORMAT.get().to_string());
            Ok(())
        }

        #[test]
        fn date_serialize_format() -> Result<(), SpanError> {
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
        fn date_deserialize_format() -> Result<(), SpanError> {
            let serialized = "{\"date\":\"2023-10-09\",\"format\":\"%d/%m/%Y\"}".to_string();
            let Ok(date) = serde_json::from_str::<Date>(&serialized) else {
                panic!("Error while deserializing date");
            };
            assert_eq!(date.to_string(), "09/10/2023".to_string());
            assert_eq!(date.format, "%d/%m/%Y".to_string());
            Ok(())
        }

        #[test]
        fn next_month_january_to_february() -> Result<(), SpanError> {
            let mut date = Date::build("2023-01-31")?;
            date = date.next(DateUnit::Month)?;
            assert_eq!(date.to_string(), "2023-02-28".to_string());
            Ok(())
        }

        #[test]
        fn next_month_february_to_march() -> Result<(), SpanError> {
            let mut date = Date::build("2023-02-02")?;
            date = date.next(DateUnit::Month)?;
            assert_eq!(date.to_string(), "2023-03-02".to_string());
            Ok(())
        }

        #[test]
        fn next_month() -> Result<(), SpanError> {
            let mut date = Date::build("2023-10-09")?;
            date = date.next(DateUnit::Month)?;
            assert_eq!(date.to_string(), "2023-11-09".to_string());
            Ok(())
        }

        #[test]
        fn next_year() -> Result<(), SpanError> {
            let mut date = Date::build("2023-10-09")?;
            date = date.next(DateUnit::Year)?;
            assert_eq!(date.to_string(), "2024-10-09".to_string());
            Ok(())
        }

        #[test]
        fn next_month_on_december() -> Result<(), SpanError> {
            let mut date = Date::build("2023-12-09")?;
            date = date.next(DateUnit::Month)?;
            assert_eq!(date.to_string(), "2024-01-09".to_string());
            Ok(())
        }

        #[test]
        fn next_day_28_february_leap_year() -> Result<(), SpanError> {
            let mut date = Date::build("2024-02-28")?;
            date = date.next(DateUnit::Day)?;
            assert_eq!(date.to_string(), "2024-02-29".to_string());
            Ok(())
        }

        #[test]
        fn next_day_28_february_non_leap_year() -> Result<(), SpanError> {
            let mut date = Date::build("2023-02-28")?;
            date = date.next(DateUnit::Day)?;
            assert_eq!(date.to_string(), "2023-03-01".to_string());
            Ok(())
        }

        #[test]
        fn matches_every_unit_in_date() -> Result<(), SpanError> {
            let date = Date::build("2023-10-09")?;
            assert!(date.matches(DateUnit::Year, 2023));
            assert!(date.matches(DateUnit::Month, 10));
            assert!(date.matches(DateUnit::Day, 9));
            Ok(())
        }

        #[test]
        fn is_in_future_yesterday() -> Result<(), SpanError> {
            let mut date = Date::today()?;
            date = date.update(DateUnit::Day, -1)?;
            assert!(!date.is_in_future()?);
            Ok(())
        }

        #[test]
        fn is_in_future_tomorrow() -> Result<(), SpanError> {
            let mut date = Date::today()?;
            date = date.update(DateUnit::Day, 1)?;
            assert!(date.is_in_future()?);
            Ok(())
        }

        #[test]
        fn is_in_future_now() -> Result<(), SpanError> {
            let date = Date::today()?;
            assert!(!date.is_in_future()?);
            Ok(())
        }

        #[test]
        fn elapsed_one_year() -> Result<(), SpanError> {
            let date = Date::build("2023-10-09")?;
            let lhs = Date::build("2022-10-09")?;
            assert_eq!(date.elapsed(&lhs), TimeDelta::try_days(365).unwrap());
            Ok(())
        }

        #[test]
        fn elapsed_one_second() -> Result<(), SpanError> {
            let date = Date::build("2023-10-09")?;
            let lhs = Date::build("2023-10-02")?;
            assert_eq!(date.elapsed(&lhs), TimeDelta::try_weeks(1).unwrap());
            Ok(())
        }

        #[test]
        fn elapsed_multiple_units() -> Result<(), SpanError> {
            let date = Date::build("2024-02-12")?;
            let lhs = Date::build("2023-02-08")?;
            assert_eq!(
                date.elapsed(&lhs),
                TimeDelta::try_weeks(52)
                    .unwrap()
                    .checked_add(&TimeDelta::try_days(5).unwrap())
                    .unwrap()
            );
            Ok(())
        }

        #[test]
        fn unit_elapsed() -> Result<(), SpanError> {
            let date = Date::build("2023-10-09")?;
            let rhs = Date::build("2020-02-08")?;
            let years_in_between = date.unit_elapsed(&rhs, DateUnit::Year)?;
            let months_in_between = date.unit_elapsed(&rhs, DateUnit::Month)?;
            let days_in_between = date.unit_elapsed(&rhs, DateUnit::Day)?;
            assert_eq!(years_in_between, 3);
            assert_eq!(months_in_between, years_in_between * 12 + 8);
            assert_eq!(days_in_between, 1338);
            Ok(())
        }

        #[test]
        fn unit_elapsed_leap_year_days() -> Result<(), SpanError> {
            let date = Date::build("2024-03-12")?;
            let rhs = Date::build("2024-01-12")?;
            let years_in_between = date.unit_elapsed(&rhs, DateUnit::Year)?;
            let months_in_between = date.unit_elapsed(&rhs, DateUnit::Month)?;
            let days_in_between = date.unit_elapsed(&rhs, DateUnit::Day)?;
            assert_eq!(years_in_between, 0);
            assert_eq!(months_in_between, years_in_between * 12 + 2);
            assert_eq!(days_in_between, 59);
            Ok(())
        }

        #[test]
        fn unit_elapsed_non_leap_year_days() -> Result<(), SpanError> {
            let date = Date::build("2023-03-12")?;
            let rhs = Date::build("2023-01-12")?;
            let years_in_between = date.unit_elapsed(&rhs, DateUnit::Year)?;
            let months_in_between = date.unit_elapsed(&rhs, DateUnit::Month)?;
            let days_in_between = date.unit_elapsed(&rhs, DateUnit::Day)?;
            assert_eq!(years_in_between, 0);
            assert_eq!(months_in_between, years_in_between * 12 + 2);
            assert_eq!(days_in_between, 58);
            Ok(())
        }
    }
}

#[cfg(all(feature = "date", feature = "datetime"))]
mod datetime_into_date {
    use crate::GetInner;

    impl From<crate::datetime::DateTime> for crate::date::Date {
        /// Convert a [DateTime] to a [Date]
        ///
        /// Only keep the date part of the [DateTime]
        ///
        /// # Example
        /// ```rust,ignore
        /// let datetime = DateTime::build("2023-10-09 12:00:00")?;
        /// let date = Date::from(datetime);
        /// assert_eq!(date.to_string(), "2023-10-09".to_string());
        /// ```
        fn from(value: crate::datetime::DateTime) -> Self {
            Self {
                date: value.date(),
                format: crate::date::BASE_DATE_FORMAT.get().to_string(),
            }
        }
    }

    #[cfg(test)]
    mod test {
        #[test]
        fn datetime_into_date() -> Result<(), crate::error::SpanError> {
            let datetime = crate::datetime::DateTime::build("2023-10-09 12:00:00")?;
            let date = crate::date::Date::from(datetime);
            assert_eq!(date.to_string(), "2023-10-09".to_string());
            Ok(())
        }

        #[test]
        #[ignore]
        fn datetime_into_date_wrong_format() -> Result<(), crate::error::SpanError> {
            let _span_builder = crate::builder::SpanBuilder::builder()
                .datetime_format("%Y-%m-%d %H:%M:%S")
                .date_format("%d/%m/%Y")
                .build();
            let datetime = crate::datetime::DateTime::build("2023-10-09 12:00:00")?;
            let date = crate::date::Date::from(datetime);
            assert_eq!(date.to_string(), "09/10/2023".to_string());
            let next = date.next(crate::date::DateUnit::Day)?;
            assert_eq!(next.to_string(), "10/10/2023".to_string());
            Ok(())
        }
    }
}
