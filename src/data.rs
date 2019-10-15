pub use crate::dimensions::{Dimensions, Weight};
pub use crate::formatted::FormattedStatus;
use chrono::{DateTime, Utc};
use iso_country::Country;
use parse_display::Display;
use serde::Deserialize;

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
    pub weight: Option<Weight>,
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

#[derive(Clone, Debug, Deserialize, Display, Eq, PartialEq)]
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

#[derive(Clone, Debug, Deserialize, Display, Eq, PartialEq)]
pub enum ReRouteAvailability {
    AvailableAfterFirstAttempt,
    CustomerRelated,
    IncorrectStatus,
}

#[derive(Clone, Debug, Deserialize, Display, Eq, PartialEq)]
pub enum PushStatus {
    Unavailable,
    On,
    Off,
}

#[derive(Clone, Debug, Deserialize, Display, Eq, PartialEq)]
pub enum DeliveryStatus {
    Delivered,
    Enroute,
    EnrouteSpecific,
    DeliveredAtPickup,
    EnrouteWholeDayOrUnspecified,
}

#[derive(Clone, Debug, Deserialize, Display, Eq, PartialEq)]
pub enum TimeFrameType {
    Specific,
    Unspecified,
    OnlyFromTime,
    WholeDay,
}

#[derive(Clone, Debug, Deserialize, Display, Eq, PartialEq)]
pub enum EnrouteType {
    Standard,
    Tentative,
}

#[derive(Clone, Debug, Deserialize, Display, Eq, PartialEq)]
pub enum ShipmentType {
    LetterboxParcel,
    Parcel,
}

#[derive(Clone, Debug, Deserialize, Display, Eq, PartialEq)]
pub enum BoxType {
    Receiver,
    Sender,
}

#[derive(Clone, Debug, Deserialize, Display, Eq, PartialEq)]
pub enum PartyType {
    Recipient,
    Return,
    Sender,
}

#[derive(Clone, Debug, Deserialize, Display, Eq, PartialEq)]
pub enum LocationType {
    Recipient,
    ServicePoint,
    Rerouted,
}
