use std::path::PathBuf;
use zatboard::coordinator::Coordinator;
use zatboard::message::Message;
use zatboard::zingo_wrapper::ZingoClient;

#[test]
fn test_full_memo_workflow() {
    let _client = ZingoClient::new(
        PathBuf::from("/tmp/test-integration"),
        "https://example.com:9067".to_string(),
    );

    let test_message = Message::new(
        "zs1sender123".to_string(),
        "zs1recipient456".to_string(),
        "ls /home".to_string(),
    );

    assert_eq!(test_message.memo_text, "ls /home");
    assert_eq!(test_message.sender_address, "zs1sender123");
    assert_eq!(test_message.recipient_address, "zs1recipient456");

    let mut signed_message = test_message.clone();
    signed_message.sign("test_key").unwrap();
    assert!(signed_message.signature.is_some());
    assert!(signed_message.verify_signature("test_key"));
}

#[test]
fn test_zatoshi_amounts() {
    let _client = ZingoClient::new(
        PathBuf::from("/tmp/test"),
        "https://example.com:9067".to_string(),
    );

    let zec_001 = 100_000;
    let zec_1 = 100_000_000;

    assert_eq!(zec_001, 100_000);
    assert_eq!(zec_1, 100_000_000);
}

#[test]
fn test_memo_command_formats() {
    let zatboard_commands = vec![
        "ls /home",
        "cat /readme.txt",
        "mkdir /new-folder",
        "chat general Hello everyone!",
    ];

    for cmd in zatboard_commands {
        let message = Message::new(
            "zs1test".to_string(),
            "zs1coordinator".to_string(),
            cmd.to_string(),
        );

        assert!(!message.memo_text.is_empty());
        assert!(message.memo_text.len() <= 512);
    }
}

#[test]
fn test_registration_and_authentication_flow() {
    let temp_dir = tempfile::tempdir().unwrap();
    let mut coordinator = Coordinator::new(
        3600,
        temp_dir.path().to_path_buf(),
        "https://example.com:9067".to_string(),
    );

    let register = Message::new(
        "zs1sender123".to_string(),
        "zs1coordinator456".to_string(),
        "REGISTER:zs1reply123".to_string(),
    );
    let register_response = coordinator.process_incoming_message(&register).unwrap();
    assert!(register_response.contains("Registration successful!"));
    assert!(register_response.contains("AUTH_CHALLENGE:"));

    let challenge = register_response
        .split("AUTH_CHALLENGE:")
        .nth(1)
        .unwrap()
        .split(' ')
        .next()
        .unwrap()
        .to_string();

    let mut auth = Message::new(
        "zs1sender123".to_string(),
        "zs1coordinator456".to_string(),
        format!("AUTH:{}", challenge),
    );
    auth.signature = Some("sig".to_string());

    let auth_response = coordinator.process_incoming_message(&auth).unwrap();
    assert!(auth_response.contains("Authentication successful"));
}

#[test]
fn test_conversation_id_command_flow() {
    let temp_dir = tempfile::tempdir().unwrap();
    let mut coordinator = Coordinator::new(
        3600,
        temp_dir.path().to_path_buf(),
        "https://example.com:9067".to_string(),
    );

    let register = Message::new(
        "zs1sender123".to_string(),
        "zs1coordinator456".to_string(),
        "REGISTER:zs1reply123".to_string(),
    );
    let register_response = coordinator.process_incoming_message(&register).unwrap();

    let conv_id = register_response
        .split("ConvID: ")
        .nth(1)
        .unwrap()
        .split(' ')
        .next()
        .unwrap()
        .to_string();
    let part_id = register_response
        .split("PartID: ")
        .nth(1)
        .unwrap()
        .split(' ')
        .next()
        .unwrap()
        .to_string();

    let command = Message::new(
        "zs1sender123".to_string(),
        "zs1coordinator456".to_string(),
        format!("{}:{}:ls /", conv_id, part_id),
    );
    let response = coordinator.process_incoming_message(&command).unwrap();
    assert!(response.contains("(empty directory)"));
}
