use std::net::{SocketAddr, TcpListener};
use std::sync::mpsc;
use tokio::time::{sleep, Duration};

mod com_with_db;
mod read_from_clients;
mod setup_client;
mod write_to_clients;

const PORT: &str = "7777";
const MSG_SIZE: usize = 64;

#[tokio::main]
async fn main() {
    let messages = com_with_db::get_messages().await;
    println!("Retrieved past messages from db");

    let local = "127.0.0.1:".to_owned() + PORT;
    let server = TcpListener::bind(local).expect("Listener failed to bind");
    server
        .set_nonblocking(true)
        .expect("Failed to initialize non-blocking");
    println!("Listening on port {}...", PORT);

    let mut clients = vec![];
    let (tx, rx) = mpsc::channel::<(SocketAddr, String)>();

    loop {
        //Check for new connections:
        if let Ok((mut socket, addr)) = server.accept() {
            println!("Client {} connected", addr);

            let tx = tx.clone();
            clients.push((addr, socket.try_clone().expect("failed to clone client")));

            let name = setup_client::get_name(&mut socket, &tx, addr);
            tokio::spawn(async move {
                read_from_clients::handle_con(&mut socket, tx, addr, name).await
            });
        }
        write_to_clients::write(&mut clients, &rx).await;
        sleep(Duration::from_millis(100)).await;
    }
}
