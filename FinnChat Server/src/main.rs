use std::net::{SocketAddr, TcpListener};
use std::sync::mpsc;
use std::thread;

mod handle_con;
mod write_to_clients;
const PORT: &str = "7777";
const MSG_SIZE: usize = 64;

fn main() {
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

            thread::spawn(move || handle_con::handle_con(&mut socket, tx, addr));
        }
        write_to_clients::write(&mut clients, &rx);

        thread::sleep(::std::time::Duration::from_millis(100))
    }
}
