use crate::common::{JsonData, Timestamp, ParseError, ProcessError, KillSwitch, Killable};
use std::{thread::{self, JoinHandle}, str, net::UdpSocket, sync::mpsc::{Sender, SendError, Receiver, self, TryRecvError}};
use log::{info, warn};

pub struct UdpHandler {
    port: i32,
    kill_tx: Option<Sender<KillSwitch>>,
}

struct UdpWorker {
    tx: Sender<JsonData>,
    kill_rx: Receiver<KillSwitch>,
    socket: UdpSocket,
}

impl UdpHandler {
    pub fn new(port: i32) -> UdpHandler {
        UdpHandler { port, kill_tx: None }
    }
    
    pub fn init(&mut self, tx: Sender<JsonData>) -> Result<JoinHandle<()>, ProcessError>{
        // Initialize UDP Socket
        let socket = match UdpSocket::bind(format!("0.0.0.0:{}", self.port)) {
            Ok(s) => s,
            Err(err) => {
                warn!("Failed to bind UDP Port: {}", err.to_string());
                return Err(ProcessError);
            }
        };
        info!("Bound to UDP Port: {}", &self.port);
        
        // Spawn a Worker Thread
        Ok(self.spawn_thread(tx, socket))
    }
    
    fn spawn_thread(&mut self, tx: Sender<JsonData>, socket: UdpSocket) -> JoinHandle<()> {
        let (kill_tx, kill_rx) = mpsc::channel();
        self.kill_tx = Some(kill_tx);
        
        let thread_handle = thread::spawn(move || {
            UdpWorker::new(tx, kill_rx, socket)
                .start();
        });
        info!("Spawned UDP Thread");
        
        thread_handle
    }
}

impl UdpWorker {
    pub fn new(tx: Sender<JsonData>, kill_rx: Receiver<KillSwitch>, socket:UdpSocket) -> UdpWorker {
        UdpWorker { tx, kill_rx, socket }
    }
    
    pub fn start(&mut self) {
        loop {
            // Check if a Kill Command Has Been Received
            match self.kill_rx.try_recv() {
                Ok(KillSwitch::ON) | Err(TryRecvError::Disconnected) => {
                    info!("Killing UDP Thread");
                    break;
                },
                Ok(KillSwitch::OFF) | Err(TryRecvError::Empty) => {},
            };
            
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

impl Killable for UdpHandler {
    fn kill(&self) {
        let tx = self.kill_tx.as_ref();
        match tx {
            Some(t) => {
                if let Err(_) = t.send(KillSwitch::ON) {
                    warn!("Failed to Kill Self");
                }
            },
            None => {
                warn!("Thread not spawned yet")
            }
        };
    }
}
