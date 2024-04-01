use std::{
    io::{BufRead, BufReader, Result, Write},
    net::{TcpListener, TcpStream},
};

fn handle(mut stream: TcpStream) -> Result<()> {
    let mut reader = BufReader::new(&stream);
    let mut buffer = String::new();

    reader.read_line(&mut buffer)?;
    let parts: Vec<&str> = buffer.split(" ").collect();
    let _method = parts[0];
    let path = parts[1];
    let _version = parts[2];

    if path == "/" {
        stream.write("HTTP/1.1 200 OK\r\n\r\n".as_bytes())?;
    } else {
        stream.write("HTTP/1.1 404 OK\r\n\r\n".as_bytes())?;
    }

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
