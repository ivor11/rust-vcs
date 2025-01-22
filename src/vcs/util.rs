use chrono::DateTime;
use chrono::offset::Utc;

pub fn systemtime_strftime<T>(dt: T, format: &str) -> String
   where T: Into<DateTime<Utc>>
{
    dt.into().format(format).to_string()
}