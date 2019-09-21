use chrono::{Date, DateTime, FixedOffset, NaiveTime, Utc};
use iso_country::Country;
use lazy_static::lazy_static;
use parse_display::Display;
use regex::{Captures, Regex};
use serde::de::{self, Deserializer};
use serde::export::TryFrom;
use serde::Deserialize;
use std::fmt;
use uom::si::f32::{Length, Mass};
use uom::si::length::{centimeter, meter};
use uom::si::mass::{gram, kilogram};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Package {
    pub key: String,
    pub sorting_key: String,
    pub title: String,
    pub sender: Option<Party>,
    pub recipient: Party,
    pub status: Status,
    pub settings: Settings,
    pub reroute: Option<ReRoute>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Address {
    #[serde(default)]
    pub is_matched: bool,
    pub street: String,
    pub house_number: String,
    pub house_number_suffix: Option<String>,
    pub postal_code: String,
    pub town: String,
    pub country: Country,
    pub formatted: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Party {
    #[serde(rename = "type")]
    pub party_type: PartyType,
    pub company_name: Option<String>,
    pub department_name: Option<String>,
    pub last_name: Option<String>,
    pub middle_name: Option<String>,
    pub first_name: Option<String>,
    pub email: Option<String>,
    pub address: Address,
    pub full_name: Option<String>,
    pub formatted: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub shipment_type: ShipmentType,
    pub barcode: String,
    pub country: String,
    pub postal_code: String,
    pub is_international: bool,
    pub web_url: String,
    pub phase: StatusPhase,
    pub enroute: Option<Enroute>,
    pub is_delivered: bool,
    pub delivery_status: DeliveryStatus,
    pub delivery_location: DeliveryLocation,
    pub delivery: Delivery,
    pub extra_information: Vec<ExtraStatusInformation>,
    pub return_eligibility: ReturnEligibility,
    pub dimensions: Option<Dimensions>,
    #[serde(deserialize_with = "deserialize_weight")]
    pub weight: Option<Mass>,
    pub formatted: Option<FormattedStatus>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Enroute {
    #[serde(rename = "timeframe")]
    pub time_frame: TimeFrame,
    #[serde(rename = "type")]
    pub enroute_type: EnrouteType,
    pub trip_information: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeFrame {
    pub planned_date: Option<DateTime<Utc>>,
    pub planned_from: Option<DateTime<Utc>>,
    pub planned_to: Option<DateTime<Utc>>,
    pub date: Option<DateTime<Utc>>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    #[serde(rename = "type")]
    pub time_frame_type: TimeFrameType,
    pub note: Option<String>,
    pub deviation_in_minutes: u32,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeliveryLocation {
    pub header: String,
    #[serde(rename = "type")]
    pub location_type: LocationType,
    pub company_name: Option<String>,
    pub department_name: Option<String>,
    pub last_name: Option<String>,
    pub middle_name: Option<String>,
    pub first_name: Option<String>,
    pub email: Option<String>,
    pub address: Address,
    pub full_name: Option<String>,
    pub formatted: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Delivery {
    pub delivery_date: Option<DateTime<Utc>>,
    pub has_proof_of_delivery: bool,
    pub signature_url: Option<String>,
    pub delivery_address: Option<Box<Address>>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReturnEligibility {
    pub can_return_at_retail: bool,
    pub pending_return_at_retail: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusPhase {
    pub index: u8,
    pub message: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReRoute {
    pub available: bool,
    pub current_selection: Option<String>,
    pub availability: ReRouteAvailability,
    pub unavailability: Option<ReRouteUnavailability>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReRouteUnavailability {
    pub text: String,
    pub link: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtraStatusInformation {
    data: ExtraStatusInformationData,
    #[serde(rename = "type")]
    information_type: ExtraStatusInformationType,
}

#[derive(Clone, Debug, Deserialize, Display)]
#[serde(rename_all = "camelCase")]
pub struct ExtraStatusInformationData {
    text: String,
}

#[derive(Clone, Debug, Deserialize, Display)]
pub enum ExtraStatusInformationType {
    Unknown,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub title: String,
    #[serde(rename = "box")]
    pub box_type: BoxType,
    pub push_notification: PushStatus,
}

#[derive(Clone, Debug, Deserialize, Display)]
pub enum ReRouteAvailability {
    AvailableAfterFirstAttempt,
    CustomerRelated,
    IncorrectStatus,
}

#[derive(Clone, Debug, Deserialize, Display)]
pub enum PushStatus {
    Unavailable,
    On,
    Off,
}

#[derive(Clone, Debug, Deserialize, Display)]
pub enum DeliveryStatus {
    Delivered,
    Enroute,
    EnrouteSpecific,
    DeliveredAtPickup,
    EnrouteWholeDayOrUnspecified,
}

#[derive(Clone, Debug, Deserialize, Display)]
pub enum TimeFrameType {
    Specific,
    Unspecified,
    OnlyFromTime,
    WholeDay,
}

#[derive(Clone, Debug, Deserialize, Display)]
pub enum EnrouteType {
    Standard,
    Tentative,
}

#[derive(Clone, Debug, Deserialize, Display)]
pub enum ShipmentType {
    LetterboxParcel,
    Parcel,
}

#[derive(Clone, Debug, Deserialize, Display)]
pub enum BoxType {
    Receiver,
    Sender,
}

#[derive(Clone, Debug, Deserialize, Display)]
pub enum PartyType {
    Recipient,
    Return,
    Sender,
}

#[derive(Clone, Debug, Deserialize, Display)]
pub enum LocationType {
    Recipient,
    ServicePoint,
    Rerouted,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(try_from = "String")]
pub struct Dimensions {
    pub height: Length,
    pub width: Length,
    pub depth: Length,
}

fn parse_float(value: &str) -> Result<f32, &'static str> {
    value
        .replace(',', ".")
        .parse()
        .map_err(|_| "Invalid formatted dimensions")
}

impl TryFrom<String> for Dimensions {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        lazy_static! {
            static ref RE: Regex =
                Regex::new(r"(^\d+(?:,\d+)?) x (\d+(?:,\d+)?) x (\d+(?:,\d+)?) (\w+)$").unwrap();
        }
        if let Some(matches) = RE.captures(&value) {
            let matches: Captures = matches;
            let h: f32 = parse_float(&matches[1])?;
            let w: f32 = parse_float(&matches[2])?;
            let d: f32 = parse_float(&matches[3])?;
            let unit = &matches[4];
            match unit {
                "cm" => Ok(Dimensions {
                    height: Length::new::<centimeter>(h),
                    width: Length::new::<centimeter>(w),
                    depth: Length::new::<centimeter>(d),
                }),
                "m" => Ok(Dimensions {
                    height: Length::new::<meter>(h),
                    width: Length::new::<meter>(w),
                    depth: Length::new::<meter>(d),
                }),
                _ => Err("Unsupported unit"),
            }
        } else {
            return Err("Invalid formatted dimensions, not matched");
        }
    }
}

pub(crate) fn deserialize_weight<'de, D>(
    deserializer: D,
) -> std::result::Result<Option<Mass>, D::Error>
where
    D: Deserializer<'de>,
{
    lazy_static! {
        static ref RE: Regex = Regex::new(r"(^\d+(?:,\d+)?) (\w+)$").unwrap();
    }

    let value = match <Option<String>>::deserialize(deserializer)? {
        Some(value) => value,
        None => return Ok(None),
    };

    if let Some(matches) = RE.captures(&value) {
        let matches: Captures = matches;
        let weight = parse_float(&matches[1]).map_err(de::Error::custom)?;
        let unit = &matches[2];
        match unit {
            "gram" => Ok(Some(Mass::new::<gram>(weight))),
            "kg" => Ok(Some(Mass::new::<kilogram>(weight))),
            _ => Err(de::Error::custom("Unsupported unit")),
        }
    } else {
        return Err(de::Error::custom("Malformed weight"));
    }
}

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
        lazy_static! {
            static ref RE: Regex = Regex::new(r"\{\}").unwrap();
        }

        params.iter().fold(format.to_string(), |result, param| {
            RE.replace(&result, param.to_string().as_str()).to_string()
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
