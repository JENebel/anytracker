pub mod writer;
pub mod reader;

use std::io::{Error, ErrorKind};

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use variant_count::VariantCount;

use crate::activity::activity_type::ActivityType;

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize, VariantCount, EnumIter)]
#[repr(u8)]
pub enum DataPoint {
    TrackPoint,
    StartSegment,
    EndSegment,
    TimeSync,
}

impl DataPoint {
    pub fn from_byte(byte: u8) -> Result<Self, Error> {
        DataPoint::iter()
            .find(|var| *var as u8 == byte)
            .ok_or(Error::new(
                ErrorKind::InvalidInput,
                format!("Failed to decode byte {byte} into DataPoint"),
            ))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Header {
    pub time: DateTime<FixedOffset>,
    pub activity_type: ActivityType,
    pub title: String,
    pub description: String,
    pub device: String,
}

pub fn encode_as_u32(input: f64) -> u32 {
    (input * (u32::MAX as f64 / 360.0) + (u32::MAX as f64 / 2.0)) as u32
}

pub fn decode_to_f64(encoded: u32) -> f64 {
    (encoded as f64 - (u32::MAX as f64 / 2.0)) / (u32::MAX as f64 / 360.0)
}

/*#[test]
fn test_serialize_time() {
    let local_time = chrono::Local::now();
    let time: DateTime<FixedOffset> = local_time.with_timezone(local_time.offset());
    println!("Before datetime: {}", time);

    let mut buffer = Vec::new();
    TrackWriter::write_fat_time(&time, &mut buffer).unwrap();
    println!("{buffer:?}");

    let (time, _) = read_fat_time(&buffer);

    println!("After datetime: {}", time);


    let buf2 = [0,0,0,0,0,0,0];
    let (time, rest) = read_fat_time(&buf2);
    println!("t2: {time}, rest: {}", rest.len())
}*/