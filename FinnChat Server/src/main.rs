use std::io::{ErrorKind, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::mpsc::{self, Sender};
use std::thread;

const PORT: &str = "7777";
const MSG_SIZE: usize = 64;

fn sleep() {
    thread::sleep(::std::time::Duration::from_millis(100));
}

fn handle_connection(socket: &mut TcpStream, tx: Sender<(SocketAddr, String)>, addr: SocketAddr) {
    //Assign Name:
    let mut name = loop {
        let mut bname = vec![0; MSG_SIZE];
        match socket.read(&mut bname) {
            Ok(_) => {
                bname = bname
                    .into_iter()
                    .take_while(|&x| x != 0)
                    .collect::<Vec<u8>>();
                let name = String::from_utf8(bname).expect("Invalid utf8 message");
                //Send Valid
                let mut is_valid = false;
                let valid = if name.len() < 3 {
                    "\\iThis name is too short!"
                } else if name.len() > 15 {
                    "\\iThis name is too long!"
                } else {
                    is_valid = true;
                    "\\v"
                };

                let mut valid = valid.to_owned().clone().into_bytes();
                valid.resize(MSG_SIZE, 0);
                socket
                    .write_all(&valid)
                    .expect("Failed to write to socket!");
                if is_valid {
                    break name;
                }
            }
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
            Err(_) => {
                println!("Client failed to provide name!")
            }
        }
    };

    //Announce Join:
    //Addr is not neccessary but is required for msg
    tx.send((addr, "\\j".to_owned() + &name + " joined!"))
        .expect("Failed to send name to rx");

    //Check for messages:
    loop {
        let mut buff = vec![0; MSG_SIZE];
        match socket.read_exact(&mut buff) {
            Ok(_) => {
                let msg = buff
                    .into_iter()
                    .take_while(|&x| x != 0)
                    .collect::<Vec<u8>>();
                let msg = String::from_utf8(msg).expect("Invalid utf8 message");
                if &msg[0..2] == "\\m" {
                    let res = (addr, "\\m".to_owned() + &name + ": " + &msg[2..]);
                    tx.send(res).expect("Failed to send msg to rx");
                } else if &msg[0..2] == "\\n" {
                    name = msg
                } else {
                    println!("Invalid packet");
                }
            }
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
            Err(_) => {
                println!("Client {} stopped responding", addr);
                break;
            }
        }

        sleep();
    }
}

fn main() {
    let local = "127.0.0.1:".to_owned() + PORT;
    let server = TcpListener::bind(local).expect("Listener failed to bind");
    server
        .set_nonblocking(true)
        .expect("failed to initialize non-blocking");
    println!("Listening on port {}...", PORT);

    let mut clients = vec![];
    let (tx, rx) = mpsc::channel::<(SocketAddr, String)>();

    loop {
        //Check for new connections:
        if let Ok((mut socket, addr)) = server.accept() {
            println!("Client {} connected", addr);

            let tx = tx.clone();
            clients.push((addr, socket.try_clone().expect("failed to clone client")));

            thread::spawn(move || handle_connection(&mut socket, tx, addr));
        }

        if let Ok((sender_addr, msg)) = rx.try_recv() {
            //Send Messages:
            if &msg[0..2] == "\\m" {
                for (addr, socket) in clients.iter_mut() {
                    if &sender_addr != addr {
                        let mut buff = msg.clone().into_bytes();
                        buff.resize(MSG_SIZE, 0);

                        socket.write_all(&buff).map(|_| socket).ok();
                    }
                }
            }
            if &msg[0..2] == "\\j" {
                for (_, socket) in clients.iter_mut() {
                    let mut buff = msg.clone().into_bytes();
                    buff.resize(MSG_SIZE, 0);

                    socket.write_all(&buff).map(|_| socket).ok();
                }
            }
        }

        sleep();
    }
}
