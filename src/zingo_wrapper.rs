use std::process::Command;
use std::path::PathBuf;

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
}