use crate::message::Message;
use crate::auth::AuthenticationFlow;
use crate::filesystem::FileSystem;
use std::collections::HashMap;
use sha2::{Sha256, Digest};

pub struct Coordinator {
    auth_flow: AuthenticationFlow,
    verified_users: HashMap<String, String>,
    pending_challenges: HashMap<String, String>,
    session_mappings: HashMap<String, String>,
    pub filesystem: FileSystem,
}

impl Coordinator {
    pub fn new(session_timeout: u64) -> Self {
        Coordinator {
            auth_flow: AuthenticationFlow::new(session_timeout),
            verified_users: HashMap::new(),
            pending_challenges: HashMap::new(),
            session_mappings: HashMap::new(),
            filesystem: FileSystem::new("coordinator".to_string()),
        }
    }

    fn handle_authenticated_command(&mut self, message: &Message) -> Result<String, String> {
        let user_id = &message.sender_address;
        
        if message.memo_text.starts_with("ls ") {
            let path = message.memo_text.strip_prefix("ls ").unwrap_or("/");
            return self.handle_ls_command(user_id, path);
        }
        
        if message.memo_text.starts_with("cat ") {
            let path = message.memo_text.strip_prefix("cat ").unwrap();
            return self.handle_cat_command(user_id, path);
        }
        
        if message.memo_text.starts_with("mkdir ") {
            let path = message.memo_text.strip_prefix("mkdir ").unwrap();
            return self.handle_mkdir_command(user_id, path);
        }
        
        if message.memo_text.starts_with("touch ") {
            let parts: Vec<&str> = message.memo_text.splitn(3, ' ').collect();
            if parts.len() >= 2 {
                let path = parts[1];
                let content = if parts.len() == 3 { parts[2] } else { "" };
                return self.handle_touch_command(user_id, path, content);
            }
        }
        
        Err("Unknown command. Try: ls <path>, cat <file>, mkdir <dir>, touch <file> [content]".to_string())
    }
    
    fn handle_ls_command(&self, user_id: &str, path: &str) -> Result<String, String> {
        let node = self.filesystem.resolve_path(path)
            .ok_or_else(|| format!("Path not found: {}", path))?;
            
        if !node.permissions.can_read(user_id) {
            return Err("Permission denied: cannot read directory".to_string());
        }
        
        if node.file_type != crate::filesystem::FileType::Directory {
            return Err("Not a directory".to_string());
        }
        
        let listing = node.list_children();
        if listing.is_empty() {
            Ok("(empty directory)".to_string())
        } else {
            Ok(listing.join("  "))
        }
    }
    
    fn handle_cat_command(&self, user_id: &str, path: &str) -> Result<String, String> {
        let node = self.filesystem.resolve_path(path)
            .ok_or_else(|| format!("File not found: {}", path))?;
            
        if !node.permissions.can_read(user_id) {
            return Err("Permission denied: cannot read file".to_string());
        }
        
        if node.file_type != crate::filesystem::FileType::File {
            return Err("Not a file".to_string());
        }
        
        Ok(node.content.clone().unwrap_or_else(|| "(empty file)".to_string()))
    }
    
    fn handle_mkdir_command(&mut self, user_id: &str, path: &str) -> Result<String, String> {
        match self.filesystem.create_directory(path, user_id.to_string()) {
            Ok(()) => Ok(format!("Directory created: {}", path)),
            Err(e) => Err(e),
        }
    }
    
    fn handle_touch_command(&mut self, user_id: &str, path: &str, content: &str) -> Result<String, String> {
        match self.filesystem.create_file(path, content.to_string(), user_id.to_string()) {
            Ok(()) => Ok(format!("File created: {}", path)),
            Err(e) => Err(e),
        }
    }

    fn generate_session_id(&self, user_address: &str) -> String {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut hasher = Sha256::new();
        hasher.update(user_address.as_bytes());
        hasher.update(timestamp.to_string().as_bytes());
        hasher.update(b"zatboard_session");

        format!("{:x}", hasher.finalize())[..16].to_string()
    }

    fn handle_authentication(&mut self, message: &Message) -> Result<String, String> {
        let parts: Vec<&str> = message.memo_text.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Err("Invalid auth format. Use AUTH:<signed_challenge>".to_string());
        }
        
        if let Some(_expected_challenge) = self.pending_challenges.get(&message.sender_address) {
            if message.signature.is_some() {
                let session_id = self.generate_session_id(&message.sender_address);
                
                let reply_address = self.auth_flow.session_manager
                    .get_reply_address(&message.sender_address)
                    .unwrap_or_else(|| message.sender_address.clone());
                
                self.verified_users.insert(message.sender_address.clone(), reply_address.clone());
                self.session_mappings.insert(session_id.clone(), reply_address);
                self.pending_challenges.remove(&message.sender_address);
                
                return Ok(format!("Authentication successful. Session ID: {}", session_id));
            }
        }
        
        Err("Authentication failed. Invalid signature or challenge.".to_string())
    }
    
    pub fn get_reply_address_by_session(&self, session_id: &str) -> Option<String> {
        self.session_mappings.get(session_id).cloned()
    }
    
    pub fn get_all_sessions(&self) -> &HashMap<String, String> {
        &self.session_mappings
    }
    
    pub fn cleanup_expired_sessions(&mut self) {
        self.auth_flow.cleanup_expired_sessions();
        // TODO: Also cleanup session_mappings based on expiry
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
    
    fn verify_sender_identity(&self, message: &Message) -> bool {
        self.verified_users.contains_key(&message.sender_address) && message.signature.is_some()
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

    #[test]
    fn test_ls_command() {
        let mut coordinator = Coordinator::new(3600);
        
        coordinator.filesystem.create_directory("/home", "coordinator".to_string()).unwrap();
        coordinator.filesystem.create_file("/home/readme.txt", "Hello!".to_string(), "coordinator".to_string()).unwrap();
        
        coordinator.verified_users.insert("zs1user123".to_string(), "zs1reply456".to_string());
        
        let ls_msg = Message::new(
            "zs1user123".to_string(),
            "zs1coordinator".to_string(),
            "ls /home".to_string()
        );
        
        let result = coordinator.handle_authenticated_command(&ls_msg);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("readme.txt"));
    }
    
    #[test]
    fn test_mkdir_command() {
        let mut coordinator = Coordinator::new(3600);
        coordinator.verified_users.insert("zs1user123".to_string(), "zs1reply456".to_string());
        coordinator.filesystem.root.permissions.add_write_permission("zs1user123".to_string());
    
        let mkdir_msg = Message::new(
            "zs1user123".to_string(),
            "zs1coordinator".to_string(),
            "mkdir /test".to_string()
        );
        
        let result = coordinator.handle_authenticated_command(&mkdir_msg);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("Directory created"));
    }


}