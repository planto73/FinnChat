use crate::com_with_db::get_messages;
use std::io::{ErrorKind, Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::sync::mpsc::Sender;

async fn verify_name(name: &String) -> Vec<u8> {
    let res = if name.len() < 3 {
        "\\iThis name is too short!".to_owned()
    } else if name.len() > 15 {
        "\\iThis name is too long!".to_owned()
    } else {
        let messages = get_messages().await;
        ["\\vWelcome!", &messages].join("\n")
    };
    let mut res = res.clone().into_bytes();
    res.resize(crate::MSG_SIZE, 0);
    res
}

async fn get_name(
    socket: &mut TcpStream,
    tx: &Sender<(SocketAddr, String)>,
    addr: SocketAddr,
) -> String {
    //Assign Name:
    let name = loop {
        let mut bname = vec![0; crate::MSG_SIZE];
        match socket.read(&mut bname) {
            Ok(_) => {
                bname = bname
                    .into_iter()
                    .take_while(|&x| x != 0)
                    .collect::<Vec<u8>>();
                let name = String::from_utf8(bname).expect("Invalid utf8 message");
                //Send Valid
                let valid = verify_name(&name).await;
                socket
                    .write_all(&valid)
                    .expect("Failed to write to socket!");
                if &valid[0..2] == "\\v".to_owned().into_bytes() {
                    break name;
                }
            }
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
            Err(_) => {
                println!("Client failed to provide name!")
            }
        }
    };
    tx.send((addr, name.clone() + " joined!"))
        .expect("Failed to send name to rx");
    name
}

pub async fn setup(
    socket: &mut TcpStream,
    tx: &Sender<(SocketAddr, String)>,
    addr: SocketAddr,
) -> String {
    let name = get_name(socket, &tx, addr).await;
    name
}
