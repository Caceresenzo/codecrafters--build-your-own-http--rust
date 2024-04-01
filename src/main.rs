use std::{
    collections::HashMap,
    env::{self, set_current_dir},
    fmt::{self, Debug},
    fs::read,
    io::{BufRead, BufReader, BufWriter, ErrorKind, Result, Write},
    net::{TcpListener, TcpStream},
    path::Path,
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
    ServerError,
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
            Status::ServerError => "500 Internal Server Error",
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

    pub fn text(status: Status, text: String) -> Response {
        let mut headers: HashMap<String, String> = HashMap::new();
        headers.insert("Content-Type".into(), "text/plain".into());
        headers.insert("Content-Length".into(), text.len().to_string());

        Response {
            status,
            headers,
            body: text.as_bytes().into(),
        }
    }

    pub fn binary(data: Vec<u8>) -> Response {
        let mut headers: HashMap<String, String> = HashMap::new();
        headers.insert("Content-Type".into(), "application/octet-stream".into());
        headers.insert("Content-Length".into(), data.len().to_string());

        Response {
            status: Status::Ok,
            headers,
            body: data,
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

        return Response::text(Status::Ok, echo.into());
    }

    if request.path.starts_with("/user-agent") {
        let user_agent = match request.headers.get("User-Agent") {
            Some(value) => value,
            None => "",
        };

        return Response::text(Status::Ok, user_agent.into());
    }

    if request.path.starts_with("/files/") {
        let path = Path::new(&request.path[7..]);

        return match read(path) {
            Ok(data) => Response::binary(data),
            Err(e) if e.kind() == ErrorKind::NotFound => Response::status(Status::NotFound),
            Err(e) => Response::text(Status::ServerError, format!("{}", e)),
        };
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

    let argv: Vec<String> = env::args().collect();
    if argv.len() == 3 {
        let path = Path::new(&argv[2]);
        assert!(set_current_dir(&path).is_ok());
        println!("changed directory: {}", path.display());
    }

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
