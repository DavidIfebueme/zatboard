use std::env;
use std::path::PathBuf;

use zatboard::message::Message;
use zatboard::zingo_wrapper::ZingoClient;

struct CliConfig {
    data_dir: PathBuf,
    server: String,
}

enum UserCommand {
    Connect { coordinator: String },
    Register { coordinator: String, reply_address: String },
    Auth { coordinator: String, challenge: String, signature: String },
    Command { coordinator: String, memo: String },
    Poll,
}

impl CliConfig {
    fn from_env() -> Self {
        let data_dir = env::var("ZATBOARD_DATA_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("./client_data"));
        let server = env::var("ZATBOARD_SERVER")
            .unwrap_or_else(|_| "http://127.0.0.1:9067".to_string());

        Self { data_dir, server }
    }
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
                return Err("Usage: zatboard register <coordinator_address> <reply_address>".to_string());
            }
            Ok(UserCommand::Register {
                coordinator: args[2].clone(),
                reply_address: args[3].clone(),
            })
        }
        "auth" => {
            if args.len() != 5 {
                return Err("Usage: zatboard auth <coordinator_address> <challenge> <signature>".to_string());
            }
            Ok(UserCommand::Auth {
                coordinator: args[2].clone(),
                challenge: args[3].clone(),
                signature: args[4].clone(),
            })
        }
        "command" => {
            if args.len() < 4 {
                return Err("Usage: zatboard command <coordinator_address> <memo_command>".to_string());
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
        .ok_or_else(|| "No wallet address found. Ensure zingo-cli wallet is initialized".to_string())
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

    match command {
        UserCommand::Connect { coordinator } => {
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
            let result = send_user_message(&client, sender, &coordinator, memo, Some("sig".to_string()))?;
            println!("{}", result.trim());
            Ok(())
        }
        UserCommand::Poll => {
            let messages = client.poll_once()?;
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
}