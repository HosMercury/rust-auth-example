use askama::Result;
use time::{macros::format_description, OffsetDateTime};

pub fn time(t: &OffsetDateTime) -> Result<String> {
    let format = format_description!("[day]-[month]-[year] [hour repr:12]:[minute] [period]");

    Ok(t.format(&format).unwrap())
}
