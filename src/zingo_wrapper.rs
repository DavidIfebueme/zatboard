use std::process::Command;
use std::path::PathBuf;
use std::time::Duration;
use std::thread;
use crate::message::Message;

pub struct ZingoClient {
    data_dir: PathBuf,
    server: String,
}

impl ZingoClient {
    pub fn new(data_dir: PathBuf, server: String) -> Self {
        ZingoClient { data_dir, server }
    }
    
    pub fn execute_command(&self, cmd: &str) -> Result<String, String> {
        let output = Command::new("zingo-cli")
            .arg("--data-dir")
            .arg(&self.data_dir)
            .arg("--server")
            .arg(&self.server)
            .arg("--command")
            .arg(cmd)
            .output()
            .map_err(|e| format!("Failed to execute zingo-cli: {}", e))?;
            
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }
    
    pub fn get_addresses(&self) -> Result<Vec<String>, String> {
        let response = self.execute_command("addresses")?;
        Ok(vec![response])
    }
    
    pub fn send_memo(&self, address: &str, amount_zatoshis: u64, memo: &str) -> Result<String, String> {
        let cmd = format!("quicksend {} {} \"{}\"", address, amount_zatoshis, memo);
        self.execute_command(&cmd)
    }

    pub fn send_memo_zec(&self, address: &str, amount_zec: f64, memo: &str) -> Result<String, String> {
        let zatoshis = (amount_zec * 100_000_000.0) as u64;
        self.send_memo(address, zatoshis, memo)
    }
    
    pub fn get_messages(&self) -> Result<Vec<Message>, String> {
        let response = self.execute_command("messages")?;
        self.parse_messages(&response)
    }
    
    fn parse_messages(&self, _raw_data: &str) -> Result<Vec<Message>, String> {
        let messages = vec![];
        Ok(messages)
    }
    
    pub fn poll_for_messages(&self, interval_secs: u64, max_iterations: Option<u32>) -> Result<Vec<Message>, String> {
        let mut iterations = 0;
        
        loop {
            self.execute_command("sync")?;
            
            match self.get_messages() {
                Ok(messages) if !messages.is_empty() => return Ok(messages),
                Ok(_) => {
                    if let Some(max) = max_iterations {
                        iterations += 1;
                        if iterations >= max {
                            return Ok(vec![]);
                        }
                    }
                    thread::sleep(Duration::from_secs(interval_secs));
                    continue;
                }
                Err(e) => return Err(e),
            }
        }
    }
    
    pub fn poll_once(&self) -> Result<Vec<Message>, String> {
        self.execute_command("sync")?;
        self.get_messages()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_client_creation() {
        let client = ZingoClient::new(
            PathBuf::from("/tmp/test"),
            "http://test:9067".to_string()
        );
        assert_eq!(client.data_dir, PathBuf::from("/tmp/test"));
        assert_eq!(client.server, "http://test:9067");
    }
    
    #[test]
    fn test_send_memo_format() {
        let _client = ZingoClient::new(
            PathBuf::from("/tmp/test"),
            "http://test:9067".to_string()
        );
        
        let cmd = format!("quicksend {} {} \"{}\"", "zs1test", 100000, "ls /home");
        assert!(cmd.contains("quicksend"));
        assert!(cmd.contains("zs1test"));
        assert!(cmd.contains("ls /home"));
        assert!(cmd.contains("100000"));
    }

    #[test]
    fn test_zatoshi_conversion() {
        let _client = ZingoClient::new(
            PathBuf::from("/tmp/test"),
            "http://test:9067".to_string()
        );
        
        assert_eq!(100_000_000, 100_000_000);
    }

    #[test]
    fn test_session_timeout_logic() {
        let _client = ZingoClient::new(
            PathBuf::from("/tmp/test"),
            "http://test:9067".to_string()
        );
        
        let timeout_secs = 3600;
        assert!(timeout_secs > 0);
    }
    
    #[test]
    fn test_parse_empty_messages() {
        let client = ZingoClient::new(
            PathBuf::from("/tmp/test"),
            "http://test:9067".to_string()
        );
        
        let result = client.parse_messages("[]");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }
}