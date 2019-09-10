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
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Address {
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
    pub is_delivered: bool,
    pub delivery_status: DeliveryStatus,
    pub delivery_location: DeliveryLocation,
    pub delivery: Delivery,
    pub return_eligibility: ReturnEligibility,
    pub dimensions: String,
    pub weight: String,
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
    pub delivery_date: DateTime<Utc>,
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
pub struct Settings {
    pub title: String,
    #[serde(rename = "box")]
    pub box_type: BoxType,
    pub push_notification: PushStatus,
}

#[derive(Clone, Debug, Deserialize, Display)]
pub enum PushStatus {
    Unavailable,
}

#[derive(Clone, Debug, Deserialize, Display)]
pub enum DeliveryStatus {
    Delivered,
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
}
