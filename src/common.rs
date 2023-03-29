use std::time::{SystemTime, UNIX_EPOCH};

use druid::{Data, Lens};
use log::warn;
use serde::{Serialize, Deserialize};

#[derive(Clone, Data, Lens)]
pub struct ClientData {
   pub udp_port: i32,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonData {
    MPU6050Data(MPU6050DataInner),
    UWBData(UWBDataInner),
}

#[derive(Serialize, Deserialize)]
pub struct MPU6050DataInner {
    source: String,
    frequency: i32,
    acc_x: f64,
    acc_y: f64,
    acc_z: f64,
    rot_x: f64,
    rot_y: f64,
    rot_z: f64,
    timestamp: Option<u64>
}

#[derive(Serialize, Deserialize)]
pub struct UWBDataInner {
    source: String,
    range: f64,
    timestamp: Option<u64>
}

pub trait Timestamp {
    fn set_timestamp(&mut self, timestamp: u64);
    
    fn set_timestamp_default(&mut self) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH);
        
        match timestamp {
            Ok(d) => self.set_timestamp(d.as_secs()),
            Err(e) => warn!("Failed to get time: {}", e.to_string())
        };
    }
}

impl ClientData {
    pub fn new(udp_port: i32) -> Self {
        ClientData {
           udp_port,
        }
    }
}

impl Timestamp for MPU6050DataInner {
    fn set_timestamp(&mut self, timestamp: u64) {
        self.timestamp = Some(timestamp);
    }    
}

impl Timestamp for UWBDataInner {
    fn set_timestamp(&mut self, timestamp: u64) {
        self.timestamp = Some(timestamp);
    }    
}