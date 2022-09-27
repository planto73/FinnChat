use std::io::{stdin, ErrorKind, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;
use std::time::Duration;

const MSG_SIZE: usize = 64;

fn input() -> String {
    loop {
        let mut buff = String::new();
        stdin()
            .read_line(&mut buff)
            .expect("Reading from stdin failed");

        if buff
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect::<String>()
            .len()
            > 0
        {
            return buff.trim().to_string();
        } else {
            println!("Must provide an input!")
        }
    }
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
                println!("{}", msg);
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

fn send_name(client: &mut TcpStream) {
    println!("What name would you like?");
    'outer: loop {
        //Send Name:
        let name = input().into_bytes();
        client.write_all(&name).expect("Writing to socket failed");
        //Receive Valid:
        loop {
            let mut buff = vec![0; MSG_SIZE];
            match client.read_exact(&mut buff) {
                Ok(_) => {
                    let valid = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                    let valid = String::from_utf8(valid).expect("Invalid utf8 messgae");

                    println!("{}", &valid[2..]);
                    if &valid[0..2] == "\\v" {
                        break 'outer;
                    } else if &valid[0..2] == "\\i" {
                        break;
                    } else {
                        println!("Invalid packet! Aborting Connection!");
                        return ();
                    }
                }
                Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
                Err(_) => {
                    println!("Connection with server was severed");
                    return ();
                }
            }
        }
    }
}

fn main() {
    //Connect to Server
    let address = get_address();
    let mut client = TcpStream::connect(address).expect("Stream failed to connect");
    client
        .set_nonblocking(true)
        .expect("Failed to initiate non-blocking");
    let (tx, rx) = mpsc::channel::<String>();
    send_name(&mut client);

    thread::spawn(move || communicate_with_server(&mut client, rx));
    //Get Message:
    println!("Write a Message or type quit");
    loop {
        let msg = "\\m".to_owned() + input().trim();
        if msg == "\\mquit" || tx.send(msg).is_err() {
            break;
        }
    }
}
