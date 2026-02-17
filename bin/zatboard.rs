use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use std::path::Path;
use zatboard::message::Message;
use zatboard::zingo_wrapper::ZingoClient;

struct CliConfig {
    data_dir: PathBuf,
    server: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
struct ClientState {
    coordinator: Option<String>,
    reply_address: Option<String>,
    conversation_id: Option<String>,
    participant_id: Option<String>,
}

enum UserCommand {
    Connect {
        coordinator: String,
    },
    Register {
        coordinator: String,
        reply_address: String,
    },
    Auth {
        coordinator: String,
        challenge: String,
        signature: String,
    },
    Command {
        coordinator: String,
        memo: String,
    },
    Poll,
}

impl CliConfig {
    fn from_env() -> Self {
        let data_dir = env::var("ZATBOARD_DATA_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("./client_data"));
        let server =
            env::var("ZATBOARD_SERVER").unwrap_or_else(|_| "http://127.0.0.1:9067".to_string());

        Self { data_dir, server }
    }
}

fn client_state_path(data_dir: &Path) -> PathBuf {
    data_dir.join("client_state.json")
}

fn load_client_state(data_dir: &Path) -> Result<ClientState, String> {
    let state_path = client_state_path(data_dir);
    if !state_path.exists() {
        return Ok(ClientState::default());
    }

    let raw = fs::read_to_string(&state_path)
        .map_err(|e| format!("Failed to read client state: {}", e))?;
    serde_json::from_str::<ClientState>(&raw)
        .map_err(|e| format!("Failed to parse client state: {}", e))
}

fn save_client_state(data_dir: &Path, state: &ClientState) -> Result<(), String> {
    fs::create_dir_all(data_dir).map_err(|e| format!("Failed to create client data dir: {}", e))?;

    let state_path = client_state_path(data_dir);
    let raw = serde_json::to_string_pretty(state)
        .map_err(|e| format!("Failed to serialize client state: {}", e))?;
    fs::write(state_path, raw).map_err(|e| format!("Failed to write client state: {}", e))
}

fn poll_with_retry(
    client: &ZingoClient,
    attempts: u8,
    delay_ms: u64,
) -> Result<Vec<Message>, String> {
    let mut last_error = None;

    for attempt in 1..=attempts.max(1) {
        match client.poll_once() {
            Ok(messages) => return Ok(messages),
            Err(e) => {
                last_error = Some(e);
                if attempt < attempts.max(1) {
                    std::thread::sleep(Duration::from_millis(delay_ms));
                }
            }
        }
    }

    Err(last_error.unwrap_or_else(|| "Polling failed".to_string()))
}

fn usage() -> &'static str {
    "ZatBoard User CLI\n\nCommands:\n  zatboard connect <coordinator_address>\n  zatboard register <coordinator_address> <reply_address>\n  zatboard auth <coordinator_address> <challenge> <signature>\n  zatboard command <coordinator_address> <memo_command>\n  zatboard poll\n\nEnvironment:\n  ZATBOARD_DATA_DIR  default ./client_data\n  ZATBOARD_SERVER    default http://127.0.0.1:9067"
}

fn parse_cli(args: &[String]) -> Result<UserCommand, String> {
    if args.len() < 2 {
        return Err(usage().to_string());
    }

    match args[1].as_str() {
        "connect" => {
            if args.len() != 3 {
                return Err("Usage: zatboard connect <coordinator_address>".to_string());
            }
            Ok(UserCommand::Connect {
                coordinator: args[2].clone(),
            })
        }
        "register" => {
            if args.len() != 4 {
                return Err(
                    "Usage: zatboard register <coordinator_address> <reply_address>".to_string(),
                );
            }
            Ok(UserCommand::Register {
                coordinator: args[2].clone(),
                reply_address: args[3].clone(),
            })
        }
        "auth" => {
            if args.len() != 5 {
                return Err(
                    "Usage: zatboard auth <coordinator_address> <challenge> <signature>"
                        .to_string(),
                );
            }
            Ok(UserCommand::Auth {
                coordinator: args[2].clone(),
                challenge: args[3].clone(),
                signature: args[4].clone(),
            })
        }
        "command" => {
            if args.len() < 4 {
                return Err(
                    "Usage: zatboard command <coordinator_address> <memo_command>".to_string(),
                );
            }
            Ok(UserCommand::Command {
                coordinator: args[2].clone(),
                memo: args[3..].join(" "),
            })
        }
        "poll" => {
            if args.len() != 2 {
                return Err("Usage: zatboard poll".to_string());
            }
            Ok(UserCommand::Poll)
        }
        _ => Err(usage().to_string()),
    }
}

fn sender_address(client: &ZingoClient) -> Result<String, String> {
    let addresses = client.get_addresses()?;
    addresses
        .into_iter()
        .find(|addr| !addr.trim().is_empty())
        .ok_or_else(|| {
            "No wallet address found. Ensure zingo-cli wallet is initialized".to_string()
        })
}

fn build_register_memo(reply_address: &str) -> String {
    format!("REGISTER:{}", reply_address)
}

fn build_auth_memo(challenge: &str) -> String {
    format!("AUTH:{}", challenge)
}

fn send_user_message(
    client: &ZingoClient,
    from: String,
    coordinator: &str,
    memo: String,
    signature: Option<String>,
) -> Result<String, String> {
    let mut message = Message::new(from, coordinator.to_string(), memo);
    message.signature = signature;
    client.send_memo(coordinator, 0, &message.memo_text)
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    let command = parse_cli(&args)?;
    let config = CliConfig::from_env();
    let client = ZingoClient::new(config.data_dir, config.server);
    let mut state = load_client_state(client.data_dir.as_path())?;

    match command {
        UserCommand::Connect { coordinator } => {
            state.coordinator = Some(coordinator.clone());
            save_client_state(client.data_dir.as_path(), &state)?;
            println!("Connected target set to {}", coordinator);
            Ok(())
        }
        UserCommand::Register {
            coordinator,
            reply_address,
        } => {
            let sender = sender_address(&client)?;
            let result = send_user_message(
                &client,
                sender,
                &coordinator,
                build_register_memo(&reply_address),
                None,
            )?;

            state.coordinator = Some(coordinator);
            state.reply_address = Some(reply_address);
            save_client_state(client.data_dir.as_path(), &state)?;

            println!("{}", result.trim());
            Ok(())
        }
        UserCommand::Auth {
            coordinator,
            challenge,
            signature,
        } => {
            let sender = sender_address(&client)?;
            let result = send_user_message(
                &client,
                sender,
                &coordinator,
                build_auth_memo(&challenge),
                Some(signature),
            )?;
            println!("{}", result.trim());
            Ok(())
        }
        UserCommand::Command { coordinator, memo } => {
            let sender = sender_address(&client)?;
            let result =
                send_user_message(&client, sender, &coordinator, memo, Some("sig".to_string()))?;
            println!("{}", result.trim());
            Ok(())
        }
        UserCommand::Poll => {
            println!("Polling for new messages...");
            let messages = poll_with_retry(&client, 3, 500)?;
            if messages.is_empty() {
                println!("No new messages.");
            }
            for msg in messages {
                println!("{}", msg);
            }
            Ok(())
        }
    }
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_register_command() {
        let args = vec![
            "zatboard".to_string(),
            "register".to_string(),
            "zs1coord".to_string(),
            "zs1reply".to_string(),
        ];

        let cmd = parse_cli(&args).unwrap();
        match cmd {
            UserCommand::Register {
                coordinator,
                reply_address,
            } => {
                assert_eq!(coordinator, "zs1coord");
                assert_eq!(reply_address, "zs1reply");
            }
            _ => panic!("Expected register command"),
        }
    }

    #[test]
    fn test_parse_auth_command() {
        let args = vec![
            "zatboard".to_string(),
            "auth".to_string(),
            "zs1coord".to_string(),
            "challenge".to_string(),
            "signature".to_string(),
        ];

        let cmd = parse_cli(&args).unwrap();
        match cmd {
            UserCommand::Auth {
                coordinator,
                challenge,
                signature,
            } => {
                assert_eq!(coordinator, "zs1coord");
                assert_eq!(challenge, "challenge");
                assert_eq!(signature, "signature");
            }
            _ => panic!("Expected auth command"),
        }
    }

    #[test]
    fn test_parse_command_with_spaces() {
        let args = vec![
            "zatboard".to_string(),
            "command".to_string(),
            "zs1coord".to_string(),
            "chat".to_string(),
            "/lobby".to_string(),
            "hello".to_string(),
            "world".to_string(),
        ];

        let cmd = parse_cli(&args).unwrap();
        match cmd {
            UserCommand::Command { coordinator, memo } => {
                assert_eq!(coordinator, "zs1coord");
                assert_eq!(memo, "chat /lobby hello world");
            }
            _ => panic!("Expected command variant"),
        }
    }

    #[test]
    fn test_parse_poll_command() {
        let args = vec!["zatboard".to_string(), "poll".to_string()];
        let cmd = parse_cli(&args).unwrap();
        assert!(matches!(cmd, UserCommand::Poll));
    }

    #[test]
    fn test_parse_invalid_command() {
        let args = vec!["zatboard".to_string(), "unknown".to_string()];
        let result = parse_cli(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_build_register_memo() {
        let memo = build_register_memo("zs1reply");
        assert_eq!(memo, "REGISTER:zs1reply");
    }

    #[test]
    fn test_build_auth_memo() {
        let memo = build_auth_memo("challenge");
        assert_eq!(memo, "AUTH:challenge");
    }

    #[test]
    fn test_state_path() {
        let path = client_state_path(PathBuf::from("/tmp/zat-test").as_path());
        assert!(path.ends_with("client_state.json"));
    }

    #[test]
    fn test_state_roundtrip() {
        let temp_dir = tempfile::tempdir().unwrap();
        let state = ClientState {
            coordinator: Some("zs1coord".to_string()),
            reply_address: Some("zs1reply".to_string()),
            conversation_id: None,
            participant_id: None,
        };

        save_client_state(temp_dir.path(), &state).unwrap();
        let loaded = load_client_state(temp_dir.path()).unwrap();
        assert_eq!(loaded, state);
    }

    #[test]
    fn test_load_state_default_when_missing() {
        let temp_dir = tempfile::tempdir().unwrap();
        let loaded = load_client_state(temp_dir.path()).unwrap();
        assert_eq!(loaded, ClientState::default());
    }
}
