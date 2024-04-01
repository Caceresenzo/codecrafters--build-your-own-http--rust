use std::{
    collections::HashMap,
    fmt::{self, Debug},
    io::{BufRead, BufReader, BufWriter, Result, Write},
    net::{TcpListener, TcpStream},
    thread,
};

#[derive(Debug)]
enum Method {
    Get,
    Post,
    Unknown,
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

#[derive(Debug)]
enum Status {
    Ok,
    // Created,
    NotFound,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

impl Status {
    fn as_str(&self) -> &'static str {
        match self {
            Status::Ok => "200 OK",
            // Status::Created => "201 Created",
            Status::NotFound => "404 Not Found",
        }
    }
}

struct Request {
    pub method: Method,
    pub path: String,
    pub version: String,
    pub headers: HashMap<String, String>,
}

struct Response {
    pub status: Status,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl Response {
    pub fn status(status: Status) -> Response {
        Response {
            status,
            headers: HashMap::new(),
            body: Vec::new(),
        }
    }

    pub fn text(text: String) -> Response {
        let mut headers: HashMap<String, String> = HashMap::new();
        headers.insert("Content-Type".into(), "text/plain".into());
        headers.insert("Content-Length".into(), text.len().to_string());

        Response {
            status: Status::Ok,
            headers,
            body: text.as_bytes().into(),
        }
    }
}

fn parse_request(reader: &mut BufReader<&TcpStream>) -> Result<Request> {
    let mut buffer = String::new();

    reader.read_line(&mut buffer)?;
    let parts: Vec<&str> = buffer.split(" ").collect();

    let method = match parts[0] {
        "GET" => Method::Get,
        "POST" => Method::Post,
        _ => Method::Unknown,
    };

    let path: String = parts[1].into();
    let version: String = parts[2].into();

    let mut headers: HashMap<String, String> = HashMap::new();

    loop {
        buffer.clear();
        reader.read_line(&mut buffer)?;
        if buffer == "\r\n" {
            break;
        }

        let index = buffer.find(':').unwrap();

        let key = &buffer[..index];
        let value = buffer[index + 1..].trim();

        headers.insert(key.into(), value.into());
    }

    Ok(Request {
        method,
        path: path.trim_end().into(),
        version: version.trim_end().into(),
        headers,
    })
}

fn answer(
    writer: &mut BufWriter<&TcpStream>,
    request: Request,
    response: Response,
) -> Result<String> {
    let space = [b' '];
    let colon = [b':', b' '];
    let crlf = [b'\r', b'\n'];

    writer.write(request.version.as_bytes())?;
    writer.write(&space)?;
    writer.write(response.status.as_str().as_bytes())?;
    writer.write(&crlf)?;

    for (key, value) in response.headers.into_iter() {
        writer.write(key.as_bytes())?;
        writer.write(&colon)?;
        writer.write(value.as_bytes())?;
        writer.write(&crlf)?;
    }

    writer.write(&crlf)?;
    writer.write(&response.body)?;

    return Ok(format!(
        "{} {} --> {}",
        request.method, request.path, response.status
    ));
}

fn route(request: &Request) -> Response {
    if request.path == "/" {
        return Response::status(Status::Ok);
    }

    if request.path.starts_with("/echo/") {
        let echo = &request.path[6..];

        return Response::text(echo.into());
    }

    if request.path.starts_with("/user-agent") {
        let user_agent = match request.headers.get("User-Agent") {
            Some(value) => value,
            None => "",
        };

        return Response::text(user_agent.into());
    }

    Response::status(Status::NotFound)
}

fn handle(stream: TcpStream) -> Result<String> {
    let mut reader = BufReader::new(&stream);
    let mut writer = BufWriter::new(&stream);

    let request = parse_request(&mut reader)?;
    let response = route(&request);

    answer(&mut writer, request, response)
}

fn main() {
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("accepted new connection");

                thread::spawn(|| match handle(stream) {
                    Ok(msg) => {
                        println!("ok: {}", msg);
                    }
                    Err(e) => {
                        println!("handle error: {}", e);
                    }
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
