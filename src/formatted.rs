use chrono::{Date, DateTime, FixedOffset, NaiveTime};
use lazy_static::lazy_static;
use regex::{Captures, Regex};
use serde::export::TryFrom;
use serde::Deserialize;
use std::fmt;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawFormattedStatus {
    title: String,
    body: String,
    short: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(try_from = "RawFormattedStatus")]
pub struct FormattedStatus {
    title: String,
    body_raw: String,
    body_params: Vec<FormattedStatusParams>,
    short_raw: String,
    short_params: Vec<FormattedStatusParams>,
}

#[derive(Clone, Debug)]
enum FormattedStatusParams {
    Date(Date<FixedOffset>),
    DateTime(DateTime<FixedOffset>),
    DateAbs(DateTime<FixedOffset>),
    Time(NaiveTime),
}

impl fmt::Display for FormattedStatusParams {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FormattedStatusParams::Date(inner) => inner.fmt(f),
            FormattedStatusParams::DateTime(inner) => inner.fmt(f),
            FormattedStatusParams::Time(inner) => inner.fmt(f),
            FormattedStatusParams::DateAbs(inner) => inner.fmt(f),
        }
    }
}

fn err_to_str(err: impl fmt::Display) -> String {
    format!("{}", err)
}

impl FormattedStatus {
    fn extract_params(raw: &str) -> Result<Vec<FormattedStatusParams>, String> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"\{(\w+):([^}]+)\}").unwrap();
        }

        let mut params = Vec::new();

        for matches in RE.captures_iter(&raw) {
            let matches: Captures = matches;

            let kind = matches[1].to_lowercase();
            let value = &matches[2];

            let parsed: FormattedStatusParams = match kind.as_str() {
                "date" => FormattedStatusParams::Date(
                    DateTime::parse_from_rfc3339(value)
                        .map_err(err_to_str)?
                        .date(),
                ),
                "time" => FormattedStatusParams::Time(
                    DateTime::parse_from_rfc3339(value)
                        .map_err(err_to_str)?
                        .time(),
                ),
                "datetime" => FormattedStatusParams::DateTime(
                    DateTime::parse_from_rfc3339(value).map_err(err_to_str)?,
                ),
                "dateabs" => FormattedStatusParams::DateAbs(
                    DateTime::parse_from_rfc3339(value).map_err(err_to_str)?,
                ),
                _ => return Err(format!("Invalid type: {}", kind)),
            };
            params.push(parsed);
        }

        Ok(params)
    }

    pub fn short(&self) -> String {
        Self::format(&self.short_raw, &self.short_params)
    }

    pub fn body(&self) -> String {
        Self::format(&self.body_raw, &self.body_params)
    }

    fn format(format: &str, params: &[FormattedStatusParams]) -> String {
        params.iter().fold(format.to_string(), |result, param| {
            result.replacen("{}", &param.to_string(), 1)
        })
    }
}

impl TryFrom<RawFormattedStatus> for FormattedStatus {
    type Error = String;

    fn try_from(value: RawFormattedStatus) -> Result<Self, Self::Error> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"\{[^}]+\}").unwrap();
        }

        let body_params = Self::extract_params(&value.body)?;
        let short_params = Self::extract_params(&value.short)?;

        Ok(FormattedStatus {
            title: value.title,
            body_raw: RE.replace_all(&value.body, "{}").to_string(),
            body_params,
            short_raw: RE.replace_all(&value.short, "{}").to_string(),
            short_params,
        })
    }
}
