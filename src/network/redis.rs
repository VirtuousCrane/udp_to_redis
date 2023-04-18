use std::{sync::mpsc::{Receiver, Sender, self, TryRecvError}, thread::{self, JoinHandle}};

use log::{warn, info};
use redis::{Connection, Client, RedisResult, Commands};

use crate::common::{JsonData, ParseError, KillSwitch, Killable};

pub struct RedisPublisherHandler {
    redis_url: String,
    redis_auth: Option<String>,
    redis_auth_pwd: Option<String>,
    kill_tx: Option<Sender<KillSwitch>>,
}

struct RedisPublisherWorker {
    rx: Receiver<JsonData>,
    kill_rx: Receiver<KillSwitch>,
    connection: Connection,
}

impl RedisPublisherHandler {
    pub fn new(redis_url: String, redis_auth: Option<String>, redis_auth_pwd: Option<String>) -> RedisPublisherHandler {
        RedisPublisherHandler {
            redis_url,
            redis_auth,
            redis_auth_pwd,
            kill_tx: None,
        }
    }
    
    pub fn init(&mut self, rx: Receiver<JsonData>) -> RedisResult<JoinHandle<()>> {
        let client = Client::open(self.redis_url.as_str())?;
        let mut connection = client.get_connection()?;
        
        if let Some(auth) = self.redis_auth.as_ref() {
            match self.redis_auth_pwd.as_ref() {
                Some(pwd) => {
                    redis::cmd("AUTH")
                        .arg(auth)
                        .arg(pwd)
                        .query(&mut connection)?;
                },
                None => {
                    redis::cmd("AUTH")
                        .arg(auth)
                        .query(&mut connection)?;
                }
            }
        }
        
        // Spawns Thread
        Ok(self.spawn_thread(rx, connection))
    }
    
    fn spawn_thread(&mut self, rx: Receiver<JsonData>, connection: Connection) -> JoinHandle<()> {
        let (kill_tx, kill_rx) = mpsc::channel();
        self.kill_tx = Some(kill_tx);
        
        let thread_handle = thread::spawn(move || {
            RedisPublisherWorker::new(rx, kill_rx, connection)
                .start();
        });
        info!("Spawned Redis Thread");
        
        thread_handle
    }
}

impl RedisPublisherWorker {
    pub fn new(rx: Receiver<JsonData>, kill_rx: Receiver<KillSwitch>, connection: Connection) -> RedisPublisherWorker {
        RedisPublisherWorker { rx, kill_rx, connection }
    }
    
    pub fn start(&mut self) {
        loop {
            // Check if a Kill Command Has Been Received
            match self.kill_rx.try_recv() {
                // Ok(KillSwitch::ON) | Err(TryRecvError::Disconnected) => {
                //     info!("Killing Redis Thread");
                //     break;
                // },
                Ok(KillSwitch::ON) => {
                    info!("KillSwitch Received: Killing Redis Thread");
                    break;
                },
                Err(TryRecvError::Disconnected) => {
                    info!("KillSwitch RX Disconnected: Killing Redis Thread");
                    break;
                },
                Ok(KillSwitch::OFF) | Err(TryRecvError::Empty) => {},
            };
            
            // Receives JSON Struct from rx
            let json_obj = match self.rx.recv() {
                Ok(obj) => obj,
                Err(e) => {
                    warn!("Failed to receive JsonData: {}", e.to_string());
                    continue;
                }
            };
            
            // Gets the channel name to publish to
            
            // CASE 1: SEPARATE BETWEEN MPU6050 AND UWB CHANNELS
            let channel = match json_obj {
                JsonData::MPU6050Data(_) => "tag:motioncapture.nodemcu",
                JsonData::UWBData(_) => "tag:motioncapture.uwb"
            };
            
            // CASE 2: LET MPU6050 AND UWB USE THE SAME CHANNEL
            // let channel = "tag:motioncapture";
            
            
            // Parses the JSON Struct into a String
            let message = match self.parse_json_to_str(&json_obj) {
                Ok(s) => s,
                Err(_) => continue,
            };
            
            // Publishes the message
            let set_result: RedisResult<()> = self.connection.set(channel, message.clone());
            let publish_result: RedisResult<()> = self.connection.publish(channel, message);

            if let Err(e) = set_result {
                warn!("Failed to set to Redis: {}", e.to_string());
                continue;
            }

            if let Err(e) = publish_result {
                warn!("Failed to publish to Redis: {}", e.to_string());
                continue;
            }

        }
    }
    
    fn parse_json_to_str(&self, json_obj: &JsonData) -> Result<String, ParseError> {
        match serde_json::to_string(json_obj) {
            Ok(s) => Ok(s),
            Err(e) => {
                warn!("Failed to parse JSON Struct into string: {}", e.to_string());
                Err(ParseError)
            }
        }
    }
}

impl Killable for RedisPublisherHandler {
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
