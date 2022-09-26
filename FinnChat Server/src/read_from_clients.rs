use std::io::{ErrorKind, Read};
use std::net::{SocketAddr, TcpStream};
use std::sync::mpsc::Sender;
use tokio::time::{sleep, Duration};

pub async fn handle_con(
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
                    let res = (addr, "\\m".to_owned() + &name + ": " + &msg[2..]);
                    tx.send(res).expect("Failed to send msg to rx");
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
        sleep(Duration::from_millis(100)).await;
    }
}
