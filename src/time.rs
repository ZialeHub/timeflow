use chrono::NaiveTime;

const BASE_TIME_FORMAT: &str = "%H:%M:%S";

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

#[derive(Debug, Clone)]
enum TimeUnit {
    Hour,
    Minute,
    Second,
}
