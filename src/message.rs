use serde::{Deserialize, Serialize};

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
    
    pub fn from_zingo_transaction(
        _transaction_data: &str
    ) -> Result<Self, String> {
        todo!("Parse from zingo-cli transaction output")
    }
    
    pub fn sign(&mut self, _private_key: &str) -> Result<(), String> {
        todo!("Implement message signing")
    }
    
    pub fn verify_signature(&self) -> bool {
        todo!("Implement signature verification")
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
}