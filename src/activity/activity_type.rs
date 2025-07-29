use std::io::{Error, ErrorKind};

use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use variant_count::VariantCount;

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize, VariantCount, EnumIter)]
#[repr(u8)]
pub enum ActivityType {
    Walk,
    Run,
    Bike,
    Swim,
    EBike,
    Motorbike,
    Car,
    Train,
    Boat,
    Plane,

    Combination,
    Unknown,
}

impl ActivityType {
    pub fn from_byte(byte: u8) -> Result<Self, Error> {
        ActivityType::iter()
            .find(|var| *var as u8 == byte)
            .ok_or(Error::new(
                ErrorKind::InvalidInput,
                format!("Failed to decode byte {byte} into ActivityType"),
            ))
    }
}
