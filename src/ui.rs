use std::{thread, str, net::UdpSocket, sync::mpsc};

use crate::{common::{ClientData, JsonData, ProcessError, Killable}, network::{udp::UdpHandler, redis::RedisPublisherHandler}, runner::spawn_handler_thread};
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
    spawn_handler_thread(data.udp_port as u32, data.redis_url.clone());
}