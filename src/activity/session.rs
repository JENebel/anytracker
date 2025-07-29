use serde::{Deserialize, Serialize};
use chrono::{DateTime, FixedOffset};
use geo::Point;
use strum_macros::EnumIter;
use variant_count::VariantCount;

use crate::activity::{activity_type::ActivityType};

#[derive(Debug, PartialEq, Eq)]
pub struct SessionId(i64);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SessionStatus {
    Live,
    Paused,
    Finished,
    /// If connection was lost, or the state is otherwise unknown
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrackSession {
    pub id: i64,
    pub time: DateTime<FixedOffset>,
    pub activity_type: ActivityType,
    pub title: String,
    pub description: String,
    pub device: String,

    /// Start time. When the device was started the first time
    pub session_status: SessionStatus,
    pub segments: Vec<Segment>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Segment {
    pub activity_type: ActivityType,
    pub start_time: DateTime<FixedOffset>,
    pub end_time: DateTime<FixedOffset>,
    pub points: Vec<TrackPoint>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrackPoint {
    point: Point,
    timestamp: DateTime<FixedOffset>,
    dynamic_data_mask: u16,
    dynamic_data: [f32; DynamicDataType::VARIANT_COUNT]
}

impl TrackPoint {
    pub fn new(point: Point, timestamp: DateTime<FixedOffset>) -> Self {
        Self {
            point,
            timestamp,
            dynamic_data_mask: 0,
            dynamic_data: [0.; DynamicDataType::VARIANT_COUNT],
        }
    }

    pub fn point(&self) -> Point {
        self.point
    }

    pub fn timestamp(&self) -> DateTime<FixedOffset> {
        self.timestamp
    }

    pub fn with_data(&mut self, data_type: DynamicDataType, data: f32) {
        self.dynamic_data_mask |= data_type.mask() as u16;
        self.dynamic_data[data_type as usize] = data;
    }

    pub fn get_dynamic_data(&self, data_type: DynamicDataType) -> Option<f32> {
        if self.dynamic_data_mask & data_type.mask() == 0 {
            return None;
        }

        Some(self.dynamic_data[data_type as usize]) 
    }

    pub (crate) fn get_dynamic_data_mask(&self) -> u16 {
        self.dynamic_data_mask
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize, VariantCount, EnumIter)]
#[repr(u8)]
pub enum DynamicDataType {
    /// Meters above sea level
    Altitude,
    /// Km/H
    Speed,
    /// Celsius
    Tempretature,
    /// BPM
    HeartRate,
    /// Steps/min
    Cadence,
    /// Watts
    Power,
    /// Total cumulative steps
    StepCount,
    /// km/liter
    FuelMilage,
    /// RPM
    RPM,
}

impl DynamicDataType {
    pub fn mask(&self) -> u16 {
        1 << *self as u16
    }
}