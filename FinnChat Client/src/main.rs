use std::error::Error;
use std::io::{stdin, ErrorKind};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

const MSG_SIZE: usize = 256;

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

async fn send_name(client: &mut TcpStream) {
    println!("What name would you like?");
    'outer: loop {
        //Send Name:
        let name = input().into_bytes();
        client.write_all(&name).await.unwrap();
        //Receive Valid:
        loop {
            let mut buff = vec![0; MSG_SIZE];
            client.readable().await.unwrap();
            match client.try_read(&mut buff) {
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

async fn read_messages(client: &mut TcpStream) {
    loop {
        //Listen for message:
        client.readable().await.unwrap();
        let mut buff = vec![0; MSG_SIZE];
        match client.try_read(&mut buff) {
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
    }
}

fn get_quit() -> bool {
    println!("Would you like to join a different server(1) or quit(2)?");
    loop {
        let quit = input();
        if quit == "1" {
            return false;
        } else if quit == "2" {
            return true;
        } else {
            println!("Please enter either 1 or 2");
        }
    }
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    'outer: loop {
        //Connect to Server
        let address = get_address();

        let std_client = std::net::TcpStream::connect(address)?;
        std_client.set_nonblocking(true)?;
        let cloned_std_client = std_client.try_clone()?;

        let mut cloned_client = TcpStream::from_std(cloned_std_client)?;
        let mut client = TcpStream::from_std(std_client)?;

        send_name(&mut client).await;
        tokio::spawn(async move { read_messages(&mut cloned_client).await });

        //Send Message:
        loop {
            println!("Write a Message or type leave");
            let msg = "\\m".to_owned() + input().trim();
            if msg == "\\mleave" {
                if get_quit() {
                    break 'outer;
                } else {
                    break;
                }
            } else {
                let mut buff = msg.clone().into_bytes();
                buff.resize(MSG_SIZE, 0);
                client.write_all(&buff).await?;
                println!("You: {}", &msg[2..]);
            }
        }
    }
    Ok(())
}
