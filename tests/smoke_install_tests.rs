use std::path::Path;
use std::process::Command;

#[test]
fn test_binary_install_smoke() {
    let bin = env!("CARGO_BIN_EXE_zatboard");
    let coordinator_bin = env!("CARGO_BIN_EXE_zatboard-coordinator");

    assert!(Path::new(bin).exists());
    assert!(Path::new(coordinator_bin).exists());

    let output = Command::new(bin)
        .output()
        .expect("failed to run zatboard binary");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("ZatBoard User CLI"));
    assert!(stderr.contains("Commands:"));
}
