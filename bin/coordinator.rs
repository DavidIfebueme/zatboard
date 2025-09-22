use zatboard::coordinator::Coordinator;
use zatboard::config::CoordinatorConfig;
use std::path::PathBuf;
use tokio;

#[tokio::main]
async fn main() {
    println!("ZatBoard Coordinator Daemon Starting...");
    
    let config_path = PathBuf::from("coordinator.toml");
    let config = match CoordinatorConfig::load_from_file(&config_path) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Error loading config: {}", e);
            std::process::exit(1);
        }
    };
    
    println!("Configuration loaded from: {}", config_path.display());
    println!("Data directory: {}", config.storage.data_dir.display());
    println!("Polling interval: {}s", config.network.polling_interval_secs);
    println!("Fees enabled: {}", config.fees.enabled);
    
    let mut coordinator = Coordinator::new(
        3600, 
        config.storage.data_dir.clone(), 
        config.network.zingo_server.clone()
    );

    if config.api.enable_json_rpc {
        println!("JSON-RPC server would start on {}:{}", config.api.bind_address, config.api.bind_port);
        // TODO: Implement JSON-RPC server properly
    }
    
    println!("Coordinator ready. Aggressive polling enabled for low latency...");
    
    loop {
        match coordinator.poll_for_new_messages() {
            Ok(messages) => {
                for message in messages {
                    if let Err(e) = coordinator.process_and_respond(&message) {
                        eprintln!("Error processing message: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error polling messages: {}", e);
                std::thread::sleep(std::time::Duration::from_secs(5));
            }
        }
        
        std::thread::sleep(std::time::Duration::from_secs(config.network.polling_interval_secs));
    }
}