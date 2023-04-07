use std::{sync::mpsc, thread};

use env_logger::{Builder, Env};
use log::info;

use crate::network::{udp::UdpHandler, redis::RedisPublisherHandler};
use crate::common::{Killable, ClientData};

/// Spawns a thread that handles UDP and Redis Connections
pub fn spawn_handler_thread(data: ClientData) {
    // Initialize Logger
    if data.verbose {
        Builder::from_env(Env::default().default_filter_or("udp_to_redis=trace"))
            .init();
        info!("Initialized Logger");
    }
    
    let auth = if data.redis_auth.is_empty() {
        None
    } else {
        Some(data.redis_auth.clone())
    };
    
    // Initialize Message Passing Channel
    let (tx, rx) = mpsc::channel();
    let mut udp_handler = UdpHandler::new(data.udp_port);
    let mut redis_handler = RedisPublisherHandler::new(data.redis_url.clone(), auth);
    
    thread::spawn(move || {
        let udp_thread_handle = match udp_handler.init(tx) {
            Ok(h) => h,
            Err(_) => return,
        };
        
        let redis_thread_handle_op = redis_handler.init(rx)
            .ok();
            
        if let None = redis_thread_handle_op {
            udp_handler.kill();
        }
        let redis_thread_handle = redis_thread_handle_op.unwrap();
        
        udp_thread_handle.join();
        redis_thread_handle.join();
    });
}
