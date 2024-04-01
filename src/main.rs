use std::{
    io::{Result, Write},
    net::{TcpListener, TcpStream},
};

fn handle(mut stream: TcpStream) -> Result<()> {
    stream.write("HTTP/1.1 200 OK\r\n\r\n".as_bytes())?;

    Ok(())
}

fn main() {
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("accepted new connection");
                
                match handle(stream) {
                    Ok(_) => {}
                    Err(e) => {
                        println!("handle error: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
