use std::path::PathBuf;
use std::process::Command;

use crate::message::Message;

pub struct ZingoClient {
    data_dir: PathBuf,
    server: String,
}

impl ZingoClient {
    pub fn new(data_dir: PathBuf, server: String) -> Self {
        ZingoClient { data_dir, server }
    }
    
    fn execute_args(&self, args: &[String]) -> Result<String, String> {
        let output = Command::new("zingo-cli")
            .arg("--data-dir")
            .arg(&self.data_dir)
            .arg("--server")
            .arg(&self.server)
            .arg("--chain")
            .arg("testnet")
            .args(args)
            .output()
            .map_err(|e| format!("Failed to execute zingo-cli: {}", e))?;
            
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            if stderr.is_empty() {
                Err("zingo-cli command failed with empty stderr".to_string())
            } else {
                Err(stderr)
            }
        }
    }

    fn split_command(cmd: &str) -> Result<Vec<String>, String> {
        let mut args = Vec::new();
        let mut current = String::new();
        let mut in_quotes = false;

        for ch in cmd.chars() {
            match ch {
                '"' => {
                    in_quotes = !in_quotes;
                }
                c if c.is_whitespace() && !in_quotes => {
                    if !current.is_empty() {
                        args.push(current.clone());
                        current.clear();
                    }
                }
                _ => current.push(ch),
            }
        }

        if in_quotes {
            return Err("Unclosed quote in command".to_string());
        }

        if !current.is_empty() {
            args.push(current);
        }

        if args.is_empty() {
            return Err("Command is empty".to_string());
        }

        Ok(args)
    }

    fn extract_json_payload(raw_data: &str) -> Option<&str> {
        let object_start = raw_data.find('{');
        let object_end = raw_data.rfind('}');
        if let (Some(start), Some(end)) = (object_start, object_end) {
            return Some(&raw_data[start..=end]);
        }

        let array_start = raw_data.find('[');
        let array_end = raw_data.rfind(']');
        if let (Some(start), Some(end)) = (array_start, array_end) {
            return Some(&raw_data[start..=end]);
        }

        None
    }

    pub fn execute_command(&self, cmd: &str) -> Result<String, String> {
        let args = Self::split_command(cmd)?;
        self.execute_args(&args)
    }
    
    pub fn get_addresses(&self) -> Result<Vec<String>, String> {
        let response = self.execute_command("addresses")?;
        let payload = Self::extract_json_payload(&response).unwrap_or(response.as_str());
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(payload) {
            if let Some(array) = value.as_array() {
                let addresses: Vec<String> = array
                    .iter()
                    .filter_map(|entry| entry.as_str().map(ToString::to_string))
                    .collect();
                if !addresses.is_empty() {
                    return Ok(addresses);
                }
            }
        }

        let addresses: Vec<String> = response
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(ToString::to_string)
            .collect();

        Ok(addresses)
    }
    
    pub fn send_memo(&self, address: &str, amount_zatoshis: u64, memo: &str) -> Result<String, String> {
        let args = vec![
            "quicksend".to_string(),
            address.to_string(),
            amount_zatoshis.to_string(),
            memo.to_string(),
        ];
        self.execute_args(&args)
    }

    pub fn send_memo_zec(&self, address: &str, amount_zec: f64, memo: &str) -> Result<String, String> {
        let zatoshis = (amount_zec * 100_000_000.0) as u64;
        self.send_memo(address, zatoshis, memo)
    }
    
    pub fn get_messages(&self) -> Result<Vec<Message>, String> {
        let response = self.execute_command("messages")?;
        self.parse_messages(&response)
    }
    
    fn parse_messages(&self, raw_data: &str) -> Result<Vec<Message>, String> {
        let json_payload = Self::extract_json_payload(raw_data)
            .ok_or_else(|| "No JSON payload found in messages response".to_string())?;

        let json = serde_json::from_str::<serde_json::Value>(json_payload)
            .map_err(|e| format!("Failed to parse messages JSON: {}", e))?;

        let mut messages = Vec::new();
        if let Some(transfers) = json.get("value_transfers").and_then(|v| v.as_array()) {
            for transfer in transfers {
                let txid = transfer
                    .get("txid")
                    .and_then(|t| t.as_str())
                    .unwrap_or("unknown_txid")
                    .to_string();

                if let Some(memos) = transfer.get("memos").and_then(|m| m.as_array()) {
                    for memo in memos {
                        if let Some(memo_text) = memo.as_str() {
                            if memo_text.is_empty() || memo_text.contains("ZecFaucet") {
                                continue;
                            }

                            let txid_prefix: String = txid.chars().take(8).collect();
                            let sender = if txid_prefix.is_empty() {
                                "client_unknown".to_string()
                            } else {
                                format!("client_{}", txid_prefix)
                            };

                            let message = Message::with_txid(
                                sender,
                                "coordinator".to_string(),
                                memo_text.to_string(),
                                txid.clone(),
                            );
                            messages.push(message);
                        }
                    }
                }
            }
        }

        Ok(messages)
    }
        
    // pub fn poll_for_new_messages(&mut self) -> Result<Vec<Message>, String> {
    //     let all_messages = self.zingo_client.poll_for_messages(1, Some(3))?;

    //     let new_messages: Vec<Message> = all_messages.into_iter()
    //         .filter(|msg| {
    //             if let Some(ref txid) = msg.txid {
    //                 if self.processed_txids.contains(txid) {
    //                     false  
    //                 } else {
    //                     self.processed_txids.insert(txid.clone()); 
    //                     true  
    //                 }
    //             } else {
    //                 true 
    //             }
    //         })
    //         .collect();
        
    //     if !new_messages.is_empty() {
    //         println!("🆕 Processing {} new messages (filtered from {})", 
    //                 new_messages.len(), 
    //                 new_messages.len() + self.processed_txids.len());
    //     }
        
    //     Ok(new_messages)
    // }
    
    pub fn poll_once(&self) -> Result<Vec<Message>, String> {
        self.execute_command("sync run")?;
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
        let args = vec![
            "quicksend".to_string(),
            "zs1test".to_string(),
            100000_u64.to_string(),
            "ls /home".to_string(),
        ];
        assert_eq!(args[0], "quicksend");
        assert_eq!(args[1], "zs1test");
        assert_eq!(args[2], "100000");
        assert_eq!(args[3], "ls /home");
    }

    #[test]
    fn test_zatoshi_conversion() {
        let _client = ZingoClient::new(
            PathBuf::from("/tmp/test"),
            "http://test:9067".to_string()
        );

        let zatoshis = (1.0_f64 * 100_000_000.0) as u64;
        assert_eq!(zatoshis, 100_000_000);
    }

    #[test]
    fn test_split_command_simple() {
        let args = ZingoClient::split_command("messages").unwrap();
        assert_eq!(args, vec!["messages".to_string()]);
    }

    #[test]
    fn test_split_command_with_quotes() {
        let args = ZingoClient::split_command("quicksend zs1abc 100 \"chat /lobby hello world\"").unwrap();
        assert_eq!(args.len(), 4);
        assert_eq!(args[0], "quicksend");
        assert_eq!(args[1], "zs1abc");
        assert_eq!(args[2], "100");
        assert_eq!(args[3], "chat /lobby hello world");
    }

    #[test]
    fn test_split_command_unclosed_quote() {
        let result = ZingoClient::split_command("quicksend zs1abc 100 \"hello");
        assert!(result.is_err());
    }
    
    #[test]
    fn test_extract_json_payload_object() {
        let raw = "noise before {\"value_transfers\":[]} noise after";
        let payload = ZingoClient::extract_json_payload(raw).unwrap();
        assert_eq!(payload, "{\"value_transfers\":[]}");
    }

    #[test]
    fn test_extract_json_payload_none() {
        assert!(ZingoClient::extract_json_payload("no json here").is_none());
    }

    #[test]
    fn test_parse_valid_messages() {
        let client = ZingoClient::new(
            PathBuf::from("/tmp/test"),
            "http://test:9067".to_string()
        );

        let raw = r#"{
            "value_transfers": [
                {
                    "txid": "abcdef1234567890",
                    "memos": ["ls /home", ""]
                }
            ]
        }"#;

        let result = client.parse_messages(raw);
        assert!(result.is_ok());
        let messages = result.unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].memo_text, "ls /home");
        assert_eq!(messages[0].sender_address, "client_abcdef12");
    }

    #[test]
    fn test_parse_messages_rejects_non_json() {
        let client = ZingoClient::new(
            PathBuf::from("/tmp/test"),
            "http://test:9067".to_string()
        );

        let result = client.parse_messages("[]");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_parse_messages_handles_short_txid() {
        let client = ZingoClient::new(
            PathBuf::from("/tmp/test"),
            "http://test:9067".to_string()
        );

        let raw = r#"{
            "value_transfers": [
                {
                    "txid": "abc",
                    "memos": ["cat /file.txt"]
                }
            ]
        }"#;

        let messages = client.parse_messages(raw).unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].sender_address, "client_abc");
    }

    #[test]
    fn test_parse_messages_filters_faucet() {
        let client = ZingoClient::new(
            PathBuf::from("/tmp/test"),
            "http://test:9067".to_string()
        );

        let raw = r#"{
            "value_transfers": [
                {
                    "txid": "abcdef1234567890",
                    "memos": ["ZecFaucet payment", "chat /lobby hi"]
                }
            ]
        }"#;

        let messages = client.parse_messages(raw).unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].memo_text, "chat /lobby hi");
    }
}