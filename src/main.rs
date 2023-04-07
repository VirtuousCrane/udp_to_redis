extern crate argparse;

use argparse::{ArgumentParser, StoreTrue, Store, StoreOption};
use common::ClientData;
use druid::{WindowDesc, AppLauncher};
use env_logger::{Builder, Env};
use log::info;

use udp_to_redis::{common, ui, runner};

fn main() {
    // Initializing the parser variables
    let mut data = ClientData::new(8888, String::from("redis://localhost:6379"), None, false);
    let mut interactive: bool = false;
    
    {
        let mut arg_parser = ArgumentParser::new();
        arg_parser.set_description("A program that accepts a UDP connection and forwards it to a Redis PubSub channel");
        
        arg_parser.refer(&mut data.verbose)
            .add_option(&["-v", "--verbose"], StoreTrue, "Shows log messages");
        
        arg_parser.refer(&mut interactive)
            .add_option(&["-i", "--interactive"], StoreTrue, "Launches a ui");
        
        arg_parser.refer(&mut data.udp_port)
            .add_option(&["-u", "--udp"], Store, "Sets the UDP Port to bind to. Defaults to 8888");
        
        arg_parser.refer(&mut data.redis_url)
            .add_option(&["-r", "--redis"], Store, "Sets the Redis Host URL to connect to. Defaults to redis://localhost:6379");
            
        arg_parser.refer(&mut data.redis_auth)
            .add_option(&["-a", "--auth"], Store, "Invokes the Redis AUTH command using the argument supplied");
        
        arg_parser.parse_args_or_exit();
    }
    
    if interactive {
        start_ui(data);
    } else {
        runner::spawn_handler_thread(data);
        loop {}
    }
    
}

fn start_ui(data: ClientData) {
    let main_window = WindowDesc::new(ui::build_ui)
        .window_size((600.0, 400.0))
        .title("UDP to Redis Worker Program");
    
    AppLauncher::with_window(main_window)
        .launch(data)
        .expect("Failed to launch program");
}