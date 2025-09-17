use crate::message::Message;
use crate::auth::AuthenticationFlow;
use std::collections::HashMap;

pub struct Coordinator {
    auth_flow: AuthenticationFlow,
    verified_users: HashMap<String, String>,
    pending_challenges: HashMap<String, String>,
}

impl Coordinator {
    pub fn new(session_timeout: u64) -> Self {
        Coordinator {
            auth_flow: AuthenticationFlow::new(session_timeout),
            verified_users: HashMap::new(),
            pending_challenges: HashMap::new(),
        }
    }
    
    pub fn process_incoming_message(&mut self, message: &Message) -> Result<String, String> {
        if message.memo_text.starts_with("REGISTER:") {
            return self.handle_registration(message);
        }
        
        if message.memo_text.starts_with("AUTH:") {
            return self.handle_authentication(message);
        }
        
        if self.verify_sender_identity(message) {
            self.handle_authenticated_command(message)
        } else {
            Err("Authentication required. Send REGISTER:<reply_address> first.".to_string())
        }
    }
    
    fn handle_registration(&mut self, message: &Message) -> Result<String, String> {
        let parts: Vec<&str> = message.memo_text.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Err("Invalid registration format. Use REGISTER:<reply_address>".to_string());
        }
        
        let reply_address = parts[1].to_string();
        let challenge = self.auth_flow.initiate_authentication(
            message.sender_address.clone(),
            reply_address.clone()
        );
        
        self.pending_challenges.insert(message.sender_address.clone(), challenge.clone());
        
        Ok(format!("Registration initiated. Please sign and send: AUTH:{}", challenge))
    }
    
    fn handle_authentication(&mut self, message: &Message) -> Result<String, String> {
        let parts: Vec<&str> = message.memo_text.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Err("Invalid auth format. Use AUTH:<signed_challenge>".to_string());
        }
        
        if let Some(expected_challenge) = self.pending_challenges.get(&message.sender_address) {
            if message.signature.is_some() {
                self.verified_users.insert(
                    message.sender_address.clone(),
                    self.auth_flow.session_manager.get_reply_address(&message.sender_address)
                        .unwrap_or_else(|| message.sender_address.clone())
                );
                self.pending_challenges.remove(&message.sender_address);
                
                return Ok("Authentication successful. You can now send commands.".to_string());
            }
        }
        
        Err("Authentication failed. Invalid signature or challenge.".to_string())
    }
    
    fn verify_sender_identity(&self, message: &Message) -> bool {
        self.verified_users.contains_key(&message.sender_address) && message.signature.is_some()
    }
    
    fn handle_authenticated_command(&mut self, message: &Message) -> Result<String, String> {
        match message.memo_text.as_str() {
            cmd if cmd.starts_with("ls ") => Ok("folder1/ folder2/ file.txt".to_string()),
            cmd if cmd.starts_with("cat ") => Ok("File contents here...".to_string()),
            cmd if cmd.starts_with("mkdir ") => Ok("Directory created successfully.".to_string()),
            _ => Err("Unknown command. Try: ls, cat, mkdir".to_string()),
        }
    }
    
    pub fn get_reply_address(&self, user_id: &str) -> Option<String> {
        self.verified_users.get(user_id).cloned()
    }
    
    pub fn is_user_verified(&self, user_id: &str) -> bool {
        self.verified_users.contains_key(user_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_coordinator_registration() {
        let mut coordinator = Coordinator::new(3600);
        
        let register_msg = Message::new(
            "zs1user123".to_string(),
            "zs1coordinator456".to_string(),
            "REGISTER:zs1reply789".to_string()
        );
        
        let result = coordinator.process_incoming_message(&register_msg);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("AUTH:"));
    }
}