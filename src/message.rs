use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub sender_address: String,
    pub recipient_address: String, 
    pub memo_text: String,
    pub signature: Option<String>,
    pub timestamp: u64,
    pub txid: Option<String>,
}

impl Message {
    pub fn new(
        sender: String,
        recipient: String, 
        memo: String
    ) -> Self {
        Message {
            sender_address: sender,
            recipient_address: recipient,
            memo_text: memo,
            signature: None,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            txid: None,
        }
    }
    
    fn create_signature_payload(&self) -> String {
        format!("{}:{}:{}:{}", 
            self.sender_address,
            self.recipient_address, 
            self.memo_text,
            self.timestamp
        )
    }
    
    pub fn sign(&mut self, private_key: &str) -> Result<(), String> {
        let payload = self.create_signature_payload();
        let signature = self.create_simple_signature(&payload, private_key);
        self.signature = Some(signature);
        Ok(())
    }
    
    pub fn verify_signature(&self, private_key: &str) -> bool {
        if let Some(ref sig) = self.signature {
            let payload = self.create_signature_payload();
            let expected = self.create_simple_signature(&payload, private_key);
            sig == &expected
        } else {
            false
        }
    }
    
    fn create_simple_signature(&self, message: &str, key: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(message.as_bytes());
        hasher.update(key.as_bytes());
        format!("{:x}", hasher.finalize())
    }
    
    pub fn from_zingo_transaction(
        _transaction_data: &str
    ) -> Result<Self, String> {
        todo!("Parse from zingo-cli transaction output")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_message_creation() {
        let msg = Message::new(
            "zs1sender123".to_string(),
            "zs1recipient456".to_string(),
            "ls /home".to_string()
        );
        
        assert_eq!(msg.memo_text, "ls /home");
        assert!(msg.timestamp > 0);
    }
    
    #[test]
    fn test_message_signing() {
        let mut msg = Message::new(
            "zs1sender123".to_string(),
            "zs1recipient456".to_string(),
            "ls /home".to_string()
        );
        
        let private_key = "test_private_key";
        msg.sign(private_key).unwrap();
        assert!(msg.signature.is_some());
        assert!(msg.verify_signature(private_key));
        
        assert!(!msg.verify_signature("wrong_key"));
    }
}