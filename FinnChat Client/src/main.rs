use std::io::{stdin, ErrorKind, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;
use std::time::Duration;

const MSG_SIZE: usize = 64;

fn input() -> String {
    let mut buff = String::new();
    stdin()
        .read_line(&mut buff)
        .expect("Reading from stdin failed");
    buff.trim().to_string()
}

fn get_address() -> String {
    println!("What host would you like to connect to?");
    let host = input();

    println!("What port would you like to connect to?");
    let port = input();

    host + ":" + &port
}

fn communicate_with_server(client: &mut TcpStream, rx: Receiver<String>) {
    loop {
        //Listen for message:
        let mut buff = vec![0; MSG_SIZE];
        match client.read_exact(&mut buff) {
            Ok(_) => {
                let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                let msg = String::from_utf8(msg).expect("Invalid utf8 messgae");
                println!("{}", &msg[2..]);
            }
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
            Err(_) => {
                println!("Connection with server was severed");
                break;
            }
        }
        //Send Message:
        match rx.try_recv() {
            Ok(msg) => {
                let mut buff = msg.clone().into_bytes();
                buff.resize(MSG_SIZE, 0);
                client.write_all(&buff).expect("Writing to socket failed");
                println!("You: {}", &msg[2..]);
            }
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => {
                break;
            }
        }

        thread::sleep(Duration::from_millis(100));
    }
}

fn main() {
    //Connect to Server
    let address = get_address();
    println!("What name would you like?");
    let name = input().into_bytes();

    let mut client = TcpStream::connect(address).expect("Stream failed to connect");
    client
        .set_nonblocking(true)
        .expect("Failed to initiate non-blocking");
    client.write_all(&name).expect("Writing to socket failed");

    let (tx, rx) = mpsc::channel::<String>();
    thread::spawn(move || communicate_with_server(&mut client, rx));
    //Get Message:
    println!("Write a Message or type quit");
    loop {
        let msg = "\\m".to_owned() + input().trim();
        if msg == "quit" || tx.send(msg).is_err() {
            break;
        }
    }
}
