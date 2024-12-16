use futures::{FutureExt, SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::{sync::Arc,  time::Duration};
use tokio::{sync::{broadcast, Mutex}, time::sleep};
use warp::{ws::{Message, WebSocket}, Filter, Rejection, Reply};

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

type CommandStore = Arc<Mutex<Vec<Command>>>;
type Clients = Arc<Mutex<Vec<broadcast::Sender<Command>>>>;

async fn handle_command(store: CommandStore) -> Result<impl Reply, Rejection> {
    let commands = store.lock().await;
    Ok(warp::reply::json(&*commands))
}

async fn add_command(
    command: Command,
    store: CommandStore,
) -> Result<impl Reply, Rejection> {
    let mut commands = store.lock().await;
    commands.push(command);
    Ok(warp::reply::json(&"Command added"))
}

async fn handle_ws_client(ws: WebSocket, clients: Clients, store: CommandStore) {
    let (client_ws_sender, mut client_ws_rcv) = ws.split();
    let (tx, _rx) = broadcast::channel(100);

    // Add client to clients list
    {
        let mut clients = clients.lock().await;
        clients.push(tx.clone());
    }

    let client_ws_sender = Arc::new(Mutex::new(client_ws_sender));
    let mut rx = tx.subscribe();

    // Handle incoming messages
    let receive_from_client = client_ws_rcv.for_each(|message| {
        let client_ws_sender = client_ws_sender.clone();
        let store = store.clone();
        
        async move {
            let message = if let Ok(msg) = message {
                msg
            } else {
                return;
            };

            let command = if let Ok(cmd) = serde_json::from_slice::<Command>(&message.as_bytes()) {
                cmd
            } else {
                return;
            };

            let mut store = store.lock().await;
            store.push(command.clone());

            // Broadcast command to all clients
            let response = serde_json::to_string(&command).unwrap();
            let mut sender = client_ws_sender.lock().await;
            let _ = sender.send(Message::text(response)).await;
        }
    });

    // Handle outgoing messages
    let client_ws_sender2 = client_ws_sender.clone();
    let receive_from_others = async move {
        while let Ok(command) = rx.recv().await {
            let response = serde_json::to_string(&command).unwrap();
            let mut sender = client_ws_sender2.lock().await;
            let _ = sender.send(Message::text(response)).await;
        }
    };

    tokio::select! {
        _ = receive_from_client => {},
        _ = receive_from_others => {},
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn create_echo_command() -> Command {
        Command {
            id: Uuid::new_v4().to_string(),
            cmd: "echo hello world".to_string(),
            status: CommandStatus::Pending,
        }
    }

    #[tokio::test]
    async fn test_add_command() {
        let store = Arc::new(Mutex::new(Vec::new()));
        let cmd = create_echo_command();
        let result = add_command(cmd.clone(), store.clone()).await;
        assert!(result.is_ok());
        
        let commands = store.lock().await;
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].cmd, "echo hello world");
    }
}

async fn periodic_test_commands(clients: Clients) {
    loop {
        let command = Command {
            id: uuid::Uuid::new_v4().to_string(),
            cmd: "echo hello world".to_string(),
            status: CommandStatus::Pending,
        };
        
        let clients = clients.lock().await;
        for client in clients.iter() {
            let _ = client.send(command.clone());
        }
        
        sleep(Duration::from_secs(3)).await;
    }
}

#[tokio::main]
async fn main() {
    let store = Arc::new(Mutex::new(Vec::new()));
    let clients = Arc::new(Mutex::new(Vec::new()));
    
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(with_clients(clients.clone()))
        .and(with_store(store.clone()))
        .map(|ws: warp::ws::Ws, clients, store| {
            ws.on_upgrade(move |socket| handle_ws_client(socket, clients, store))
        });

    // Start periodic test command sender
    let test_clients = clients.clone();
    tokio::spawn(async move {
        periodic_test_commands(test_clients).await;
    });

    println!("WebSocket server started at ws://localhost:3030/ws");
    warp::serve(ws_route).run(([127, 0, 0, 1], 3030)).await;
}

fn with_clients(clients: Clients) -> impl Filter<Extract = (Clients,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || clients.clone())
}

fn with_store(store: CommandStore) -> impl Filter<Extract = (CommandStore,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || store.clone())
}
