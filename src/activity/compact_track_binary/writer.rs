use std::{
    io::{Error, Write},
    marker::PhantomData,
};

use chrono::{DateTime, FixedOffset};
use strum::IntoEnumIterator;

use crate::activity::{
    compact_track_binary::{encode_as_u32, DataPoint, Header},
    session::{DynamicDataType, Segment, TrackPoint},
};

pub struct TrackWriter<W: Write> {
    phantom_data: PhantomData<W>,
    reference_time: DateTime<FixedOffset>,
}

impl<W: Write> TrackWriter<W> {
    pub fn start(
        time: DateTime<FixedOffset>,
        header: Header,

        writer: &mut W,
    ) -> Result<(Self, usize), Error> {
        // Write header
        let mut written = Self::write_fat_time(&time, writer)?;
        written += writer.write(&[header.activity_type as u8])?;
        written += Self::write_string(&header.title, writer)?;
        written += Self::write_string(&header.description, writer)?;
        written += Self::write_string(&header.device, writer)?;

        Ok((
            Self {
                phantom_data: PhantomData,
                reference_time: time,
            },
            written,
        ))
    }

    /// Will crop string to max 255 bytes, which should be plenty enough
    /// Might be a cutting unicode chars, corrupting the string, but this is handled in decoding
    pub fn write_string(string: &str, writer: &mut W) -> Result<usize, Error> {
        let string_bytes = string.as_bytes();
        let bytes_to_write = string_bytes.len().min(255);
        let mut written = writer.write(&[bytes_to_write as u8])?;
        written += writer.write(&string_bytes[..bytes_to_write])?;

        Ok(written)
    }

    pub fn write_segment(&mut self, segment: &Segment, writer: &mut W) -> Result<usize, Error> {
        let mut written = 0;

        // Start segment
        written += Self::write_data_point(DataPoint::StartSegment, writer)?;
        written += Self::write_fat_time(&segment.start_time, writer)?;
        written += writer.write(&[segment.activity_type as u8])?;

        self.reference_time = segment.start_time;

        // Write points
        for track_point in &segment.points {
            written += self.write_track_point(track_point, writer)?;
        }

        // End segment
        written += Self::write_data_point(DataPoint::EndSegment, writer)?;
        written += Self::write_fat_time(&segment.start_time, writer)?;

        Ok(written)
    }

    fn write_data_point(data_point: DataPoint, writer: &mut W) -> Result<usize, Error> {
        writer.write(&[data_point as u8])
    }

    pub fn write_track_point(
        &mut self,
        track_point: &TrackPoint,
        writer: &mut W,
    ) -> Result<usize, Error> {
        let mut written = 0;

        // Add time sync point if needed
        let mut time_diff_secs = track_point
            .timestamp()
            .signed_duration_since(self.reference_time)
            .as_seconds_f64();

        if time_diff_secs > u8::MAX as f64 {
            // Add a time sync
            self.reference_time = track_point.timestamp();
            written += Self::write_data_point(DataPoint::TimeSync, writer)?;
            written += Self::write_fat_time(&track_point.timestamp(), writer)?;
            time_diff_secs = 0.;
        }

        Self::write_data_point(DataPoint::TrackPoint, writer)?;

        // Lat/lon
        written += writer.write(&encode_as_u32(track_point.point().x()).to_be_bytes())?;
        written += writer.write(&encode_as_u32(track_point.point().y()).to_be_bytes())?;

        // Thin time stamp
        written += writer.write(&(time_diff_secs as u8).to_be_bytes())?;

        // Write dynamic data
        let mask = track_point.get_dynamic_data_mask();
        written += writer.write(&mask.to_be_bytes())?;
        for data_type in DynamicDataType::iter() {
            if let Some(data) = track_point.get_dynamic_data(data_type) {
                written += writer.write(&data.to_be_bytes())?;
            }
        }

        Ok(written)
    }

    // 7 bytes: 5 timestamp + 2 offset
    pub(super) fn write_fat_time(
        time: &DateTime<FixedOffset>,
        writer: &mut W,
    ) -> Result<usize, Error> {
        let utc_timestamp_secs = (time.timestamp_millis() / 1000) as u64;
        let offset_mins = (time.offset().local_minus_utc() / 60) as u16;

        let mut written = writer.write(&utc_timestamp_secs.to_be_bytes()[3..])?;
        written += writer.write(&offset_mins.to_be_bytes())?;
        Ok(written)
    }
}
