use chrono::{DateTime, Utc};
use iso_country::Country;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Package {
    key: String,
    sorting_key: String,
    title: String,
    sender: Option<Party>,
    recipient: Party,
    status: Status,
    settings: Settings,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Address {
    is_matched: bool,
    street: String,
    house_number: String,
    house_number_suffix: Option<String>,
    postal_code: String,
    town: String,
    country: Country,
    formatted: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Party {
    #[serde(rename = "type")]
    party_type: PartyType,
    company_name: Option<String>,
    department_name: Option<String>,
    last_name: Option<String>,
    middle_name: Option<String>,
    first_name: Option<String>,
    email: Option<String>,
    address: Address,
    full_name: Option<String>,
    formatted: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    shipment_type: ShipmentType,
    barcode: String,
    country: String,
    postal_code: String,
    is_international: bool,
    web_url: String,
    phase: StatusPhase,
    is_delivered: bool,
    delivery_status: DeliveryStatus,
    delivery_location: DeliveryLocation,
    delivery: Delivery,
    return_eligibility: ReturnEligibility,
    dimensions: String,
    weight: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeliveryLocation {
    header: String,
    #[serde(rename = "type")]
    location_type: LocationType,
    company_name: Option<String>,
    department_name: Option<String>,
    last_name: Option<String>,
    middle_name: Option<String>,
    first_name: Option<String>,
    email: Option<String>,
    address: Address,
    full_name: Option<String>,
    formatted: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Delivery {
    delivery_date: DateTime<Utc>,
    has_proof_of_delivery: bool,
    signature_url: Option<String>,
    delivery_address: Option<Box<Address>>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReturnEligibility {
    can_return_at_retail: bool,
    pending_return_at_retail: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusPhase {
    index: u8,
    message: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    title: String,
    #[serde(rename = "box")]
    box_type: BoxType,
    push_notification: PushStatus,
}

#[derive(Clone, Debug, Deserialize)]
pub enum PushStatus {
    Unavailable,
}

#[derive(Clone, Debug, Deserialize)]
pub enum DeliveryStatus {
    Delivered,
}

#[derive(Clone, Debug, Deserialize)]
pub enum ShipmentType {
    LetterboxParcel,
    Parcel,
}

#[derive(Clone, Debug, Deserialize)]
pub enum BoxType {
    Receiver,
    Sender,
}

#[derive(Clone, Debug, Deserialize)]
pub enum PartyType {
    Recipient,
    Return,
    Sender,
}

#[derive(Clone, Debug, Deserialize)]
pub enum LocationType {
    Recipient,
    ServicePoint,
}
