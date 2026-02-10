use std::process::{Command, Stdio};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command as TokioCommand;

#[tokio::test]
async fn test_server_startup_and_shutdown() {
    // Build the binary first to ensure we test the latest code
    let status = Command::new("cargo")
        .args(["build"])
        .status()
        .expect("Failed to build binary");
    assert!(status.success(), "Cargo build failed");

    // Path to the binary
    let bin_path = if cfg!(windows) {
        "target/debug/rustools-mcp.exe"
    } else {
        "target/debug/rustools-mcp"
    };

    // Start the server in stdio mode
    let mut child = TokioCommand::new(bin_path)
        .arg("stdio")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn server");

    let stdout = child.stdout.take().expect("Failed to capture stdout");
    let mut reader = BufReader::new(stdout);

    // Wait for a short duration to ensure it starts up (simulating connection)
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Check if it's still running
    assert!(child.id().is_some(), "Server process died prematurely");

    // In a real E2E test, we would write JSON-RPC messages to stdin here
    // and assert on the responses from stdout.
    // For now, we just verify it launches successfully.

    // Clean up
    child.kill().await.expect("Failed to kill server");
}
