use std::{
    io::{Error, Read},
    marker::PhantomData,
};

use chrono::{DateTime, FixedOffset};

use crate::activity::{activity_type::ActivityType, compact_track_binary::Header};

pub struct TrackReader<R: Read> {
    phantom_data: PhantomData<R>,
    reference_time: DateTime<FixedOffset>,
}

impl<R: Read> TrackReader<R> {
    /// Will read the header, and return a reader that can read data points
    pub fn start_reading(reader: &mut R) -> Result<(Self, Header), Error> {
        // Read header
        let time = Self::read_fat_time(reader)?;
        let activity_type = Self::read_activity_type(reader)?;
        let title = Self::read_string(reader)?;
        let description = Self::read_string(reader)?;
        let device = Self::read_string(reader)?;

        let header = Header {
            time,
            activity_type,
            title,
            description,
            device,
        };

        Ok((
            Self {
                phantom_data: PhantomData,
                reference_time: time,
            },
            header,
        ))
    }

    fn read_activity_type(reader: &mut R) -> Result<ActivityType, Error> {
        let mut buf = [0];
        reader.read_exact(&mut buf)?;
        ActivityType::from_byte(buf[0])
    }

    fn read_string(reader: &mut R) -> Result<String, Error> {
        let mut buf = [0];
        reader.read_exact(&mut buf)?;
        let mut string_buf = vec![0u8; buf[0] as usize];
        reader.read_exact(&mut string_buf)?;
        Ok(String::from_utf8_lossy(&string_buf).to_string())
    }

    fn read_fat_time(reader: &mut R) -> Result<DateTime<FixedOffset>, Error> {
        let mut utc_timestamp_secs_bytes = [0; 8];
        reader.read_exact(&mut utc_timestamp_secs_bytes[3..])?;

        let mut offset_bytes = [0; 2];
        reader.read_exact(&mut offset_bytes[0..])?;

        let utc_timestamp_secs = u64::from_be_bytes(utc_timestamp_secs_bytes) as i64;
        let offset_mins = u16::from_be_bytes(offset_bytes) as i32;

        let tz = FixedOffset::east_opt(offset_mins * 60).unwrap();
        let res = DateTime::from_timestamp(utc_timestamp_secs, 0)
            .unwrap()
            .naive_utc()
            .and_local_timezone(tz)
            .unwrap();

        Ok(res)
    }
}
