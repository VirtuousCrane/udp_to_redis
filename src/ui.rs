use std::{thread, str, net::UdpSocket, sync::mpsc};

use crate::{common::{ClientData, JsonData, ProcessError, Killable}, network::{udp::UdpHandler, redis::RedisPublisherHandler}};
use druid::{Widget, widget::{Container, Label, Flex, LensWrap, TextBox, Button}, text::format::ParseFormatter, WidgetExt, EventCtx, Env};
use log::{info, warn};

/// Creates the ui of the program
pub fn build_ui() -> impl Widget<ClientData> {
    let redis_data_input_row = Flex::row()
        .with_child(Label::new("Redis URL: "))
        .with_child(LensWrap::new(
            TextBox::new()
                .with_formatter(ParseFormatter::new()),
            ClientData::redis_url
        ));
    
   let udp_data_input_row = Flex::row()
        .with_child(Label::new("UDP Port: "))
        .with_child(LensWrap::new(
            TextBox::new()
                .with_formatter(ParseFormatter::new()),
            ClientData::udp_port
        ))
        .with_spacer(10.0)
        .with_child(
            Button::new("Connect")
                .on_click(button_callback)
        );
    
    Container::new(
        Flex::column()
            .with_child(redis_data_input_row)
            .with_child(udp_data_input_row)
            .center()
    )
}

/// Callback for the connect button. Creates a new thread to listen to UDP messages
fn button_callback(_ctx: &mut EventCtx, data: &mut ClientData, _env: &Env) {
    // Initialize Message Passing Channel
    let (tx, rx) = mpsc::channel();
    let mut udp_handler = UdpHandler::new(data.udp_port);
    let mut redis_handler = RedisPublisherHandler::new(&data.redis_url);
    
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
