pub mod date;
pub mod datetime;
pub mod error;
pub mod prelude;
pub mod time;

#[cfg(test)]
pub mod test {
    use super::*;
    use prelude::*;

    #[test]
    fn test_log_error_datetime_parse() {
        let datetime = DateTime::build("09-10-2023 00_00_00");
        let _ = datetime.inspect_err(|e| {
            assert_eq!(
                e.to_string(),
                "DateTime ➤  ParseFromStr: input contains invalid characters"
            )
        });
    }

    #[test]
    fn test_log_error_datetime_parse_timestamp() {
        let err = SpanError::DateTime(
            Box::new(SpanError::ParseFromTimestamp(
                "parse_from_timestamp error".to_string(),
            )),
            DateTimeError,
        );
        assert_eq!(
            err.to_string(),
            "DateTime ➤  ParseFromTimestamp: parse_from_timestamp error"
        )
    }

    #[test]
    fn test_log_error_datetime_invalid_update() {
        let err = SpanError::DateTime(
            Box::new(SpanError::InvalidUpdate(
                "Cannot add x Day to datetime error".to_string(),
            )),
            DateTimeError,
        );
        assert_eq!(
            err.to_string(),
            "DateTime ➤  InvalidUpdate: Cannot add x Day to datetime error"
        )
    }

    #[test]
    fn test_log_error_date_parse() {
        let datetime = Date::build("10-31-2023");
        let _ = datetime.inspect_err(|e| {
            assert_eq!(e.to_string(), "Date ➤  ParseFromStr: input is out of range")
        });
    }

    #[test]
    fn test_log_error_date_invalid_update() {
        let err = SpanError::Date(
            Box::new(SpanError::InvalidUpdate(
                "Cannot add x Month to date error".to_string(),
            )),
            DateError,
        );
        assert_eq!(
            err.to_string(),
            "Date ➤  InvalidUpdate: Cannot add x Month to date error"
        )
    }

    #[test]
    fn test_log_error_time_parse() {
        let time = Time::build("00_00_00");
        let _ = time.inspect_err(|e| {
            assert_eq!(
                e.to_string(),
                "Time ➤  ParseFromStr: input contains invalid characters"
            )
        });
    }

    #[test]
    fn test_log_error_time_invalid_update() {
        let err = SpanError::Time(
            Box::new(SpanError::InvalidUpdate(
                "Cannot add x Second to time error".to_string(),
            )),
            TimeError,
        );
        assert_eq!(
            err.to_string(),
            "Time ➤  InvalidUpdate: Cannot add x Second to time error"
        )
    }
}
