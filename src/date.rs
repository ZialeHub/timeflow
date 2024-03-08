use chrono::NaiveDate;

const BASE_DATE_FORMAT: &str = "%Y-%m-%d";

/// Date structure to handle date management
///
/// const BASE_DATE_FORMAT: &str = "%Y-%m-%d";
///
/// BASE_DATE_FORMAT is the default format for date
// TODO Find a way to implement Serialize and Deserialize for Time
#[derive(Debug, Clone)]
struct DateTime {
    pub datetime: NaiveDate,
    pub format: String,
}

#[derive(Debug, Clone)]
enum DateUnit {
    Year,
    Month,
    Day,
}
