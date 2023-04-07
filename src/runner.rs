use std::{sync::mpsc, thread};

use crate::network::{udp::UdpHandler, redis::RedisPublisherHandler};
use crate::common::Killable;

/// Spawns a thread that handles UDP and Redis Connections
pub fn spawn_handler_thread(udp_port: u32, redis_url: String) {
    // Initialize Message Passing Channel
    let (tx, rx) = mpsc::channel();
    let mut udp_handler = UdpHandler::new(udp_port);
    let mut redis_handler = RedisPublisherHandler::new(&redis_url);
    
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
