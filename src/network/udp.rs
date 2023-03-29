use crate::common::{JsonData, Timestamp};
use core::fmt;
use std::{thread::{self, JoinHandle}, str, net::UdpSocket, sync::mpsc::{Sender, SendError}, error::Error};
use log::{info, warn};

pub struct UdpHandler {
    port: i32,
}

#[derive(Debug)]
pub struct ParseError;

struct UdpWorker {
    tx: Sender<JsonData>,
    socket: UdpSocket,
}

impl UdpHandler {
    pub fn new(port: i32) -> UdpHandler {
        UdpHandler { port }
    }
    
    pub fn init(&self, tx: Sender<JsonData>) {
        // Initialize UDP Socket
        let socket = match UdpSocket::bind(format!("0.0.0.0:{}", self.port)) {
            Ok(s) => s,
            Err(err) => {
                warn!("Failed to bind UDP Port: {}", err.to_string());
                return;
            }
        };
        info!("Bound to UDP Port: {}", &self.port);
        
        // Spawn a Worker Thread
        self.spawn_thread(tx, socket);
    }
    
    fn spawn_thread(&self, tx: Sender<JsonData>, socket: UdpSocket) -> JoinHandle<()> {
        thread::spawn(move || {
            UdpWorker::new(tx, socket)
                .start();
        })
    }
}

impl UdpWorker {
    pub fn new(tx: Sender<JsonData>, socket:UdpSocket) -> UdpWorker {
        UdpWorker { tx, socket }
    }
    
    pub fn start(&mut self) {
        loop {
            // Receives UDP Message
            let mut msg = [0; 512];
            if let Err(err) = self.socket.recv(&mut msg) {
                warn!("Failed to receive UDP message: {}", err.to_string());
            }
            
            // Parse UDP Message into String
            let msg_str = match self.parse_msg_to_str(&msg) {
                Ok(m) => m,
                Err(_) => continue,
            };
            
            // Parse Message String as JSON Struct
            let mut json_data = match self.parse_msg_str_as_json(&msg_str) {
                Ok(j) => j,
                Err(_) => continue,
            };
            self.add_timestamp_to_json(&mut json_data);
            
            // Passes Json Struct to Consumer Thread
            if let Err(_) = self.send_message(json_data) {
                warn!("Failed to Pass Message Object to Redis");
                continue;
            }
        }
    }
    
    fn parse_msg_to_str(&self, buf: &[u8; 512]) -> Result<String, ParseError> {
        match str::from_utf8(buf) {
            Ok(res) => {
                info!("Received: {}", res.trim());
                let result = res.trim_matches(char::from(0));
                Ok(result.trim().into())
            },
            Err(e) => {
                warn!("{}", e.to_string());
                Err(ParseError)
            },
        }
    }
    
    fn parse_msg_str_as_json(&self, msg: &str) -> Result<JsonData, ParseError> {
        match serde_json::from_str::<JsonData>(msg) {
            Ok(obj) => {
                Ok(obj)
            },
            Err(e) => {
                warn!("Failed to parse String into JsonData: {}", e);
                Err(ParseError)
            }
        }
    }
    
    fn add_timestamp_to_json(&self, data: &mut JsonData) {
        match data {
            JsonData::UWBData(u) => u.set_timestamp_default(),
            JsonData::MPU6050Data(m) => m.set_timestamp_default(),
        };
    }
    
    fn send_message(&self, json_data: JsonData) -> Result<(), SendError<JsonData>> {
        self.tx.send(json_data)
    }
}

impl Error for ParseError {}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Failed to Parse String")
    }
}