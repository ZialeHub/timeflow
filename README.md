[![Rust CI](https://github.com/ryse-rs/span/actions/workflows/ci-main.yaml/badge.svg)](https://github.com/ryse-rs/span/actions/workflows/ci-main.yaml)
[![codecov](https://codecov.io/gh/ryse-rs/span/graph/badge.svg?token=E7HLJRBTXZ)](https://codecov.io/gh/ryse-rs/span)

# What is span?

Span is a rust time management library. It encapsulates commonly used libs such as `chrono` and adds up nice features.

Span allows to create, update, compare and display custom formats of `Time`, `Date` or `Datetime` with ease.

Each enum represents a different unit of time:
- `TimeUnit::Hour/Minute/Second`
- `DateUnit::Year/Month/Day`
- `DateTimeUnit::Year/Month/Day/Hour/Minute/Second`

# Builder

We provide a `SpanBuilder` to set a custom date format. This leads to less boilerplate for each date calls, and improved consistency through the entire application.
Formats are defined based on a `strftime` inspired date and time formatting syntax.

Here is the default format for each type:
- Time => `"%H:%M:%S"`
- Date => `"%Y-%m-%d"`
- DateTime => `format!("{} {}", BASE_DATE_FORMAT, BASE_TIME_FORMAT)`

# Features

By default _span_ can be used to manage `time`, `date` or `datetime`, but you're free to select features for your app.

- (Default)`["full"]`
- `["time"]`
- `["date"]`
- `["datetime"]`

# Examples

```rust
let _span = crate::builder::SpanBuilder::builder()
    .date_format("%d %m %Y")
    .time_format("T%H:%M:%SZ.000")
    .datetime_format("%Y-%m-%d %H:%M:%S")
    .build();

let mut time = Time::build("T23:17:12Z.000")?;

time.update(TimeUnit::Hour, 1)?;
assert_eq!(time.to_string(), "T00:17:12Z.000");

let mut datetime = DateTime::build("2024-10-31 06:32:28")?;

datetime.update(DateTimeUnit::Month, 1)?;
assert_eq!(datetime.to_string(), "2024-11-30 06:32:28");

assert!(datetime.matches(DateTimeUnit::Year, 2024));
assert!(datetime.matches(DateTimeUnit::Hour, 6));
assert!(datetime.matches(DateTimeUnit::Day, 30));

let previous_datetime = DateTime::build("2022-01-22 06:32:28")?;
let elapsed = datetime.elapsed(&previous_datetime);
assert_eq!(elapsed, TimeDelta::try_days(1043).unwrap());

let year_elapsed = datetime.unit_elapsed(DateTimeUnit::Year, &previous_datetime);
assert_eq!(year_elapsed, 2);

let is_in_future = previous_datetime.is_in_future()?;
assert!(!is_in_future);

eprintln!("DateTime == '{}'", datetime);
// "DateTime == '2024-11-30 06:32:28'"
```
