use std::net::SocketAddr;

use futures::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::{Framed, LinesCodec, LinesCodecError};
/*
sudo tcpdump -i any port 65534 -n -S -vvv
iptables -A OUTPUT -p tcp --tcp-flags RST RST -j DROP
*/
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tcp_listener = TcpListener::bind("127.0.0.1:65534").await?;
    println!("Server listen on {}", &tcp_listener.local_addr()?);

    loop {
        let (stream, addr) = tcp_listener.accept().await?;

        tokio::spawn(async move {
            println!("Client {} connect.", &addr);
            process_client(stream, &addr).await;
        });
    }
}
async fn process_client(stream: TcpStream, addr: &SocketAddr) {
    let mut framed = Framed::new(stream, LinesCodec::new());
    if let Err(e) = framed.send("Hi, I am a server cat.").await {
        eprintln!("Error: {} when hello to client: {}", e, addr);
    }

    loop {
        let message = match framed.next().await {
            Some(Ok(msg)) => {
                println!("Client {} send: {}", &addr, &msg);
                msg
            }
            Some(Err(err)) => {
                match err {
                    LinesCodecError::MaxLineLengthExceeded => {
                        println!("Client {} send a too long message.", &addr);
                        framed.send("Your message is too long.").await.unwrap();
                        break;
                    }

                    _ => {
                        println!("Client {} has an error: {}.", &addr, &err);
                        break;
                    }
                }
            }
            None => {
                println!("Client {} disconnect.", &addr);
                break;
            }
        };

        match message.as_str() {
            "close" => {
                println!("Client {} disconnect.", &addr);
                framed.send("Bye bye.").await.unwrap();
                break;
            }

            _ => {
                framed.send(format!("Server received: {}", &message)).await.unwrap();
            }
        }
    }
}