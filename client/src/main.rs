use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::process::Command as ProcessCommand;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tokio::time::{sleep, Duration};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Command {
    id: String,
    cmd: String,
    status: CommandStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum CommandStatus {
    Pending,
    Completed(String),
    Failed(String),
}

async fn execute_command(cmd: &str) -> CommandStatus {
    let output = ProcessCommand::new("sh")
        .arg("-c")
        .arg(cmd)
        .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                CommandStatus::Completed(String::from_utf8_lossy(&output.stdout).to_string())
            } else {
                CommandStatus::Failed(String::from_utf8_lossy(&output.stderr).to_string())
            }
        }
        Err(e) => CommandStatus::Failed(e.to_string()),
    }
}

#[tokio::main]
async fn main() {
    let url = url::Url::parse("ws://localhost:3030/ws").unwrap();
    
    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    let (mut write, mut read) = ws_stream.split();
    
    println!("WebSocket Client connected");

    while let Some(message) = read.next().await {
        if let Ok(message) = message {
            if let Ok(command) = serde_json::from_str::<Command>(&message.to_string()) {
                if matches!(command.status, CommandStatus::Pending) {
                    println!("Executing command: {}", command.cmd);
                    let result = execute_command(&command.cmd).await;
                    
                    // Send result back to server
                    let completed_command = Command {
                        id: command.id,
                        cmd: command.cmd,
                        status: result,
                    };
                    
                    let message = serde_json::to_string(&completed_command).unwrap();
                    write.send(Message::Text(message)).await.unwrap();
                }
            }
        }
    }
}
