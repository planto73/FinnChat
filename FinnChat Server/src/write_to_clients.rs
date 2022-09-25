use std::io::Write;
use std::net::{SocketAddr, TcpStream};
use std::sync::mpsc::Receiver;

pub fn write(clients: &mut Vec<(SocketAddr, TcpStream)>, rx: &Receiver<(SocketAddr, String)>) {
    if let Ok((sender_addr, msg)) = rx.try_recv() {
        //Send Messages:
        if &msg[0..2] == "\\m" {
            for (addr, socket) in clients.iter_mut() {
                if &sender_addr != addr {
                    let mut buff = msg.clone().into_bytes();
                    buff.resize(crate::MSG_SIZE, 0);

                    socket.write_all(&buff).map(|_| socket).ok();
                }
            }
        }
        if &msg[0..2] == "\\j" {
            for (_, socket) in clients.iter_mut() {
                let mut buff = msg.clone().into_bytes();
                buff.resize(crate::MSG_SIZE, 0);

                socket.write_all(&buff).map(|_| socket).ok();
            }
        }
    }
}
