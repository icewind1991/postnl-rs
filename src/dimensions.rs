use lazy_static::lazy_static;
use regex::{Captures, Regex};
use serde::export::TryFrom;
use serde::Deserialize;
use uom::si::f32::{Length, Mass};
use uom::si::length::{centimeter, meter};
use uom::si::mass::{gram, kilogram};

#[derive(Clone, Debug, Deserialize, PartialEq)]
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
            height: Length::new::<centimeter>(21.0),
            width: Length::new::<centimeter>(30.0),
            depth: Length::new::<centimeter>(40.5),
        },
        dimensions
    );

    let input = "2 x 1 x 1 m".to_string();
    let dimensions: Dimensions = input.try_into().unwrap();

    assert_eq!(
        Dimensions {
            height: Length::new::<meter>(2.0),
            width: Length::new::<meter>(1.0),
            depth: Length::new::<meter>(1.0),
        },
        dimensions
    );
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(try_from = "String")]
pub struct Weight(Mass);

impl TryFrom<String> for Weight {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"(^\d+(?:,\d+)?) (\w+)$").unwrap();
        }

        if let Some(matches) = RE.captures(&value) {
            let matches: Captures = matches;
            let weight = parse_float(&matches[1])?;
            let unit = &matches[2];
            match unit {
                "gram" => Ok(Weight(Mass::new::<gram>(weight))),
                "kg" => Ok(Weight(Mass::new::<kilogram>(weight))),
                _ => Err("Unsupported unit"),
            }
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

    assert_eq!(Weight(Mass::new::<kilogram>(3.0)), weight);

    let input = "300 gram".to_string();
    let weight: Weight = input.try_into().unwrap();

    assert_eq!(Weight(Mass::new::<gram>(300.0)), weight);
}
