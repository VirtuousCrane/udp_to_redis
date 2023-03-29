use std::{thread, str, net::UdpSocket, sync::mpsc};

use crate::{common::{ClientData, JsonData}, network::udp::UdpHandler};
use druid::{Widget, widget::{Container, Label, Flex, LensWrap, TextBox, Button}, text::format::ParseFormatter, WidgetExt, EventCtx, Env};
use log::{info, warn};

/// Creates the ui of the program
pub fn build_ui() -> impl Widget<ClientData> {
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
            .with_child(udp_data_input_row)
            .center()
    )
}

/// Callback for the connect button. Creates a new thread to listen to UDP messages
fn button_callback(_ctx: &mut EventCtx, data: &mut ClientData, _env: &Env) {
    // Initialize Message Passing Channel
    let (tx, rx) = mpsc::channel();
    let udp_handler = UdpHandler::new(data.udp_port);
    
    udp_handler.init(tx);
    
    let _redis_thread_handle = thread::spawn(move || {
        loop {
            let payload = match rx.recv() {
                Ok(msg) => msg,
                Err(_) => continue,
            };
            
            let topic = match payload {
                JsonData::MPU6050Data { .. } => {
                    "MOTIONCAPTURE/MPU6050"
                },
                JsonData::UWBData { .. } => {
                    "MOTIONCAPTURE/UWB"
                }
            };
            
            let payload_str = match serde_json::to_string(&payload) {
                Ok(s) => s,
                Err(_) => {
                    warn!("Failed to parse Payload into String");
                    return;
                },
            };
       }
    });
}
