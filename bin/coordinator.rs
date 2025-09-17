use zatboard::coordinator::Coordinator;

fn main() {
    println!("🚀 ZatBoard Coordinator Daemon Starting...");
    
    let mut coordinator = Coordinator::new(3600); // 1 hour session timeout
    
    // TODO: In Phase 7, we'll add:
    // - Config file loading
    // - Zingo-cli integration  
    // - Message polling loop
    // - Response sending
    
    println!("✅ Coordinator ready. Listening for messages...");
    println!("📋 Supported commands: REGISTER, AUTH, ls, cat, mkdir");
    
    // For now, just demonstrate the coordinator logic
    demo_coordinator_logic(&mut coordinator);
}

fn demo_coordinator_logic(coordinator: &mut Coordinator) {
    use zatboard::message::Message;
    
    println!("\n🧪 Demo: User Registration Flow");
    
    // Simulate user registration
    let register_msg = Message::new(
        "zs1user123".to_string(),
        "zs1coordinator456".to_string(),
        "REGISTER:zs1reply789".to_string()
    );
    
    match coordinator.process_incoming_message(&register_msg) {
        Ok(response) => println!("📤 Coordinator response: {}", response),
        Err(error) => println!("❌ Error: {}", error),
    }
    
    println!("\n🧪 Demo: Unauthenticated Command (should fail)");
    
    let unauth_msg = Message::new(
        "zs1user123".to_string(),
        "zs1coordinator456".to_string(),
        "ls /home".to_string()
    );
    
    match coordinator.process_incoming_message(&unauth_msg) {
        Ok(response) => println!("📤 Coordinator response: {}", response),
        Err(error) => println!("❌ Expected error: {}", error),
    }
}