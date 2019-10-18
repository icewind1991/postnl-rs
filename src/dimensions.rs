use once_cell::sync::Lazy;
use parse_display::Display;
use regex::Regex;
use serde::export::TryFrom;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, PartialEq, Display)]
#[serde(try_from = "String")]
#[display("{height} x {width} x {depth}m")]
pub struct Dimensions {
    pub height: f32,
    pub width: f32,
    pub depth: f32,
}

fn parse_float(value: &str) -> Result<f32, &'static str> {
    value
        .replace(',', ".")
        .parse()
        .map_err(|_| "Invalid formatted dimensions")
}

static DIMENSIONS_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(^\d+(?:,\d+)?) x (\d+(?:,\d+)?) x (\d+(?:,\d+)?) (\w+)$").unwrap());

impl TryFrom<String> for Dimensions {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if let Some(matches) = DIMENSIONS_REGEX.captures(&value) {
            let h: f32 = parse_float(&matches[1])?;
            let w: f32 = parse_float(&matches[2])?;
            let d: f32 = parse_float(&matches[3])?;
            let unit = &matches[4];
            let multiplier = match unit {
                "cm" => 100.0,
                "m" => 1.0,
                _ => return Err("Unsupported unit"),
            };
            Ok(Dimensions {
                height: h / multiplier,
                width: w / multiplier,
                depth: d / multiplier,
            })
        } else {
            Err("Invalid formatted dimensions, not matched")
        }
    }
}

#[test]
fn test_parse_dimensions() {
    use std::convert::TryInto;

    let input = "21 x 30 x 40,5 cm".to_string();
    let dimensions: Dimensions = input.try_into().unwrap();

    assert_eq!(
        Dimensions {
            height: 0.21,
            width: 0.3,
            depth: 0.405
        },
        dimensions
    );

    let input = "2 x 1 x 1 m".to_string();
    let dimensions: Dimensions = input.try_into().unwrap();

    assert_eq!(
        Dimensions {
            height: 2.0,
            width: 1.0,
            depth: 1.0
        },
        dimensions
    );
}

#[derive(Clone, Debug, PartialEq, Deserialize, Display)]
#[serde(try_from = "String")]
#[display("{0}kg")]
pub struct Weight(f32);

static WEIGHT_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(^\d+(?:,\d+)?) (\w+)$").unwrap());

impl TryFrom<String> for Weight {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if let Some(matches) = WEIGHT_REGEX.captures(&value) {
            let weight = parse_float(&matches[1])?;
            let unit = &matches[2];
            let multiplier = match unit {
                "gram" => 1000.0,
                "kg" => 1.0,
                _ => return Err("Unsupported unit"),
            };
            Ok(Weight(weight / multiplier))
        } else {
            Err("Malformed weight")
        }
    }
}

#[test]
fn test_parse_weight() {
    use std::convert::TryInto;

    let input = "3 kg".to_string();
    let weight: Weight = input.try_into().unwrap();

    assert_eq!(Weight(3.0), weight);

    let input = "300 gram".to_string();
    let weight: Weight = input.try_into().unwrap();

    assert_eq!(Weight(0.3), weight);
}
