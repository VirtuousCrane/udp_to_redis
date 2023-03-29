use common::ClientData;
use druid::{WindowDesc, AppLauncher};
use env_logger::{Builder, Env};
use log::info;

mod ui;
mod common;

fn main() {
    // Initialize Logger
    Builder::from_env(Env::default().default_filter_or("udp_to_redis=trace"))
        .init();
    info!("Initialized Logger");
    
    let main_window = WindowDesc::new(ui::build_ui)
        .window_size((600.0, 400.0))
        .title("UDP to Redis Worker Program");
    
    let initial_data = ClientData::new(8888);
    
    AppLauncher::with_window(main_window)
        .launch(initial_data)
        .expect("Failed to launch program");
}    
