use std::io::{ErrorKind, Read};
use std::net::{SocketAddr, TcpStream};
use std::sync::mpsc::Sender;
use tokio::time::{sleep, Duration};

use crate::com_with_db;

pub async fn read(
    socket: &mut TcpStream,
    tx: Sender<(SocketAddr, String)>,
    addr: SocketAddr,
    name: String,
) {
    //Check for messages:
    loop {
        let mut buff = vec![0; crate::MSG_SIZE];
        match socket.read_exact(&mut buff) {
            Ok(_) => {
                let msg = buff
                    .into_iter()
                    .take_while(|&x| x != 0)
                    .collect::<Vec<u8>>();
                let msg = String::from_utf8(msg).expect("Invalid utf8 message");

                if &msg[0..2] == "\\m" {
                    com_with_db::update_db(name.clone(), &msg[2..]).await;

                    let res = (addr, name.clone() + ": " + &msg[2..]);
                    tx.send(res).expect("Failed to send msg to rx");
                } else {
                    println!("Client {} sent an invalid packet!", addr);
                    tx.send((addr, name + " left!"))
                        .expect("Failed to send msg to rx");
                    return;
                }
            }
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
            Err(_) => {
                println!("Client {} stopped responding!", addr);
                tx.send((addr, name + " left!"))
                    .expect("Failed to send msg to rx");
                return;
            }
        }
        sleep(Duration::from_millis(100)).await;
    }
}
