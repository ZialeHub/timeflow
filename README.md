[![ci](https://github.com/ZialeHub/span/actions/workflows/ci.yml/badge.svg)](https://github.com/ZialeHub/span/actions/workflows/ci.yml)

##### ❓ What is it?

Span is a rust library designed to make time management easy.  
It encapsulates commonly used libs such as `chrono` and adds up nice features.

Span allows to create, update, compare and display custom formats of `Time`, `Date` or `Datetime` with ease, while setting default de/serialization and timezone options.

##### 🔭 What is our vision for this project?

Replace the usage of any other time-related library.

##### 🚨 What problem does it solve?

Time management is hard: setting up, operations, de/serialization, timezones... Span provides a simple API relying on the builder pattern, to easily perform all imaginable operations.

##### 🎯 Who is it for?

Developers.

- Whenever you want to perform time-related operations.
- Whenever you want to use serde on time-related strings and fields.
- As a full replacement for the `chrono` library.


## ✨ Features

By default _span_ can be used to manage `time`, `date` or `datetime`, but you're free to select features for your app.

- (Default)`["full"]`
- `["time"]`
- `["date"]`
- `["datetime"]`

## 🚀 Usage

Run `cargo add span` to your crate.

Each enum represents a different unit of time:
- `TimeUnit::Hour/Minute/Second`
- `DateUnit::Year/Month/Day`
- `DateTimeUnit::Year/Month/Day/Hour/Minute/Second`

#### Builder

We provide a `SpanBuilder` to set a custom date format. This leads to less boilerplate for each date calls, and improved consistency through the entire application.
Formats are defined based on a `strftime` inspired date and time formatting syntax.

Here is the default format for each type:
- Time => `"%H:%M:%S"`
- Date => `"%Y-%m-%d"`
- DateTime => `format!("{} {}", BASE_DATE_FORMAT, BASE_TIME_FORMAT)`

## 👀 Examples

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

## 🤝 Contributing

Please always perform the following checks before committing:  
1. ⚙️ `cargo build --workspace --all --all-features --tests`
2. 🧼 `cargo fmt --all`
3. 🩺 `cargo clippy --workspace --all --all-features --tests -- -D warnings`
4. 🧪 `cargo test --all-targets --all-features --workspace`

## 📄 License

This project is licensed under the MIT License. See LICENSE for details.

## Contributors

Huge thanks to [gpoblon](https://github.com/gpoblon) for his CI, ideas and reviews!
