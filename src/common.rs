use std::{time::{SystemTime, UNIX_EPOCH}, error::Error, fmt};

use druid::{Data, Lens};
use log::warn;
use serde::{Serialize, Deserialize};

#[derive(Clone, Data, Lens)]
pub struct ClientData {
   pub udp_port: i32,
   pub redis_url: String,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonData {
    MPU6050Data(MPU6050DataInner),
    UWBData(UWBDataInner),
}

pub enum KillSwitch {
    OFF,
    ON,
}

#[derive(Serialize, Deserialize)]
pub struct MPU6050DataInner {
    pub source: String,
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
    pub source: String,
    range: f64,
    timestamp: Option<u64>
}

#[derive(Debug)]
pub struct ParseError;

#[derive(Debug)]
pub struct ProcessError;

pub trait Killable {
    fn kill(&self) {}
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
            redis_url: String::from("redis://127.0.0.1/"),
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

impl Error for ParseError {}
impl Error for ProcessError {}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Failed to Parse String")
    }
}

impl fmt::Display for ProcessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Failed to Start Process")
    }
}