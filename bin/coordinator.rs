use std::path::PathBuf;
use zatboard::config::CoordinatorConfig;
use zatboard::coordinator::Coordinator;

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
    println!(
        "Polling interval: {}s",
        config.network.polling_interval_secs
    );
    println!("Fees enabled: {}", config.fees.enabled);

    let mut coordinator = Coordinator::new_with_options(
        3600,
        config.storage.data_dir.clone(),
        config.network.zingo_server.clone(),
        config.storage.database_file.clone(),
        config.storage.cache_ttl_secs,
    );

    if config.api.enable_json_rpc {
        println!(
            "JSON-RPC server starting on {}:{}",
            config.api.bind_address, config.api.bind_port
        );
        let rpc_coordinator = Coordinator::new_with_options(
            3600,
            config.storage.data_dir.clone(),
            config.network.zingo_server.clone(),
            config.storage.database_file.clone(),
            config.storage.cache_ttl_secs,
        );
        let bind_address = config.api.bind_address.clone();
        let bind_port = config.api.bind_port;

        tokio::spawn(async move {
            if let Err(e) = rpc_coordinator
                .start_json_rpc_server(bind_address, bind_port)
                .await
            {
                eprintln!("❌ JSON-RPC server failed: {}", e);
            }
        });
    }

    println!("Coordinator ready. Aggressive polling enabled for low latency...");

    loop {
        match coordinator.poll_for_new_messages() {
            Ok(messages) => {
                if messages.is_empty() {
                    std::thread::sleep(std::time::Duration::from_secs(5));
                    continue;
                }

                for message in messages {
                    match coordinator.process_and_respond(&message) {
                        Ok(()) => println!("📤 Message processed successfully"),
                        Err(e) => eprintln!("❌ Error processing message: {}", e),
                    }
                }
            }
            Err(e) => {
                eprintln!("⚠️  Error polling messages: {}", e);
                std::thread::sleep(std::time::Duration::from_secs(5));
            }
        }

        std::thread::sleep(std::time::Duration::from_secs(
            config.network.polling_interval_secs,
        ));
    }
}
