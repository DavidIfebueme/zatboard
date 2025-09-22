use zatboard::coordinator::Coordinator;
use std::path::PathBuf;

fn main() {
    println!("ZatBoard Coordinator Daemon Starting...");
    
    let zingo_data_dir = PathBuf::from("./coordinator_data");
    let zingo_server = "http://localhost:9067".to_string();
    
    let mut coordinator = Coordinator::new(3600, zingo_data_dir, zingo_server);
    
    println!("Coordinator ready. Will respond via Zcash memos...");
    
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
                std::thread::sleep(std::time::Duration::from_secs(10));
            }
        }
        
        std::thread::sleep(std::time::Duration::from_secs(5));
    }
}