use anyhow::Error;
use owo_colors::OwoColorize;
use rockbox_sys as rb;
use serde::Serialize;
use serde_json::Value;
use sqlx::Sqlite;
use std::{
    collections::HashMap,
    future::Future,
    io::{BufRead, BufReader, Read, Write},
    net::{TcpListener, TcpStream},
    pin::Pin,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use threadpool::ThreadPool;

type Handler = fn(&Context, &Request, &mut Response);

pub struct Context {
    pub pool: sqlx::Pool<Sqlite>,
}

#[derive(Debug)]
pub struct Request {
    pub method: String,
    pub params: Vec<String>,
    pub query_params: Value,
    pub body: Option<String>,
}

#[derive(Debug)]
pub struct Response {
    body: String,
    status_code: u16,
    headers: HashMap<String, String>,
}

impl Response {
    pub fn new() -> Self {
        Response {
            body: String::new(),
            status_code: 200,
            headers: HashMap::new(),
        }
    }

    pub fn json<T: Serialize>(&mut self, value: &T) {
        let json_value = serde_json::to_value(value).unwrap();
        self.add_header("Content-Type", "application/json");
        self.body = serde_json::to_string(&json_value).unwrap();
    }

    pub fn text(&mut self, text: &str) {
        self.add_header("Content-Type", "text/plain");
        self.body = text.to_string();
    }

    pub fn set_body(&mut self, body: &str) {
        self.body = body.to_string();
    }

    pub fn set_status(&mut self, status: u16) {
        self.status_code = status;
    }

    pub fn add_header(&mut self, key: &str, value: &str) {
        self.headers.insert(key.to_string(), value.to_string());
    }

    pub fn send(self, stream: &mut TcpStream) {
        let status_line = format!("HTTP/1.1 {} OK\r\n", self.status_code);
        let mut response = status_line;

        // Add headers
        for (key, value) in self.headers {
            response.push_str(&format!("{}: {}\r\n", key, value));
        }

        // Add content length header
        response.push_str(&format!("Content-Length: {}\r\n", self.body.len()));

        // End headers
        response.push_str("\r\n");

        // Add body
        response.push_str(&self.body);

        // Write response to the stream
        stream.write_all(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    }
}

fn split_path_and_query(path: &str) -> (&str, Option<&str>) {
    match path.find('?') {
        Some(pos) => (&path[..pos], Some(&path[pos + 1..])),
        None => (path, None),
    }
}

#[derive(Clone)]
struct Router {
    routes: HashMap<String, HashMap<String, Handler>>, // method -> path -> handler
}

impl Router {
    pub fn new() -> Self {
        Router {
            routes: HashMap::new(),
        }
    }

    // Define the `get` method for routing GET requests
    fn get(&mut self, path: &str, handler: Handler) {
        self.add_route("GET", path, handler);
    }

    // Define the `post` method for routing POST requests
    fn post(&mut self, path: &str, handler: Handler) {
        self.add_route("POST", path, handler);
    }

    // Define the `put` method for routing PUT requests
    fn put(&mut self, path: &str, handler: Handler) {
        self.add_route("PUT", path, handler);
    }

    fn delete(&mut self, path: &str, handler: Handler) {
        self.add_route("DELETE", path, handler);
    }

    // Add route to the routing table
    pub fn add_route(&mut self, method: &str, path: &str, handler: Handler) {
        self.routes
            .entry(method.to_string())
            .or_insert_with(HashMap::new)
            .insert(path.to_string(), handler);
    }

    // Match the method and path to find the corresponding handler
    pub fn route(&self, method: &str, path: &str) -> Option<(&Handler, Vec<String>)> {
        let (path_without_query, _) = split_path_and_query(path);
        if let Some(routes) = self.routes.get(method) {
            for (route_path, handler) in routes {
                let mut params = Vec::new();
                if self.match_route(route_path, path_without_query, &mut params) {
                    return Some((handler, params));
                }
            }
        }
        None
    }

    // Simple route matching to support dynamic parameters
    pub fn match_route(
        &self,
        route_path: &str,
        request_path: &str,
        params: &mut Vec<String>,
    ) -> bool {
        let route_parts: Vec<&str> = route_path.split('/').collect();
        let request_parts: Vec<&str> = request_path.split('/').collect();

        if route_parts.len() > request_parts.len() {
            return false;
        }

        for (route_part, request_part) in route_parts.iter().zip(request_parts.iter()) {
            if route_part.starts_with(":") {
                params.push(request_part.to_string()); // Capture the parameter
            } else if route_part != request_part {
                return false; // Paths don't match
            }
        }

        // Ensure that the remaining parts of the request path are empty if the route path is shorter
        if route_parts.len() < request_parts.len() {
            for remaining_part in &request_parts[route_parts.len()..] {
                if !remaining_part.is_empty() {
                    return false;
                }
            }
        }

        true
    }
}

#[derive(Clone)]
pub struct RockboxHttpServer {
    router: Router,
}

impl RockboxHttpServer {
    pub fn new() -> Self {
        RockboxHttpServer {
            router: Router::new(),
        }
    }

    // Define the `get` method for routing GET requests
    pub fn get(&mut self, path: &str, handler: Handler) {
        self.router.get(path, handler);
    }

    // Define the `post` method for routing POST requests
    pub fn post(&mut self, path: &str, handler: Handler) {
        self.router.post(path, handler);
    }

    // Define the `put` method for routing PUT requests
    pub fn put(&mut self, path: &str, handler: Handler) {
        self.router.put(path, handler);
    }

    // Define the `delete` method for routing DELETE requests
    pub fn delete(&mut self, path: &str, handler: Handler) {
        self.router.delete(path, handler);
    }

    // Start listening and handling incoming requests
    pub fn listen(&mut self) -> Result<(), Error> {
        const BANNER: &str = r#"
                  __________               __   ___.
        Open      \______   \ ____   ____ |  | _\_ |__   _______  ___
        Source     |       _//  _ \_/ ___\|  |/ /| __ \ /  _ \  \/  /
        Jukebox    |    |   (  <_> )  \___|    < | \_\ (  <_> > <  <
        Firmware   |____|_  /\____/ \___  >__|_ \|___  /\____/__/\_ \
                          \/            \/     \/    \/            \/
       "#;

        println!("{}", BANNER.yellow());

        let port = std::env::var("ROCKBOX_TCP_PORT").unwrap_or_else(|_| "6063".to_string());
        let addr = format!("0.0.0.0:{}", port);
        let listener = TcpListener::bind(&addr)?;
        listener.set_nonblocking(true)?;

        println!(
            "{} server is running on {}",
            "Rockbox TCP".bright_purple(),
            addr.bright_green()
        );

        let pool = ThreadPool::new(4);
        let active_connections = Arc::new(Mutex::new(0));
        let rt = tokio::runtime::Runtime::new()?;
        let db_pool = rt.block_on(rockbox_library::create_connection_pool())?;

        loop {
            match listener.accept() {
                Ok((stream, _)) => {
                    let db_pool = db_pool.clone();
                    let active_connections = Arc::clone(&active_connections);
                    {
                        let mut active_connections = active_connections.lock().unwrap();
                        *active_connections += 1;
                    }
                    let mut cloned_self = self.clone();
                    pool.execute(move || {
                        let mut buf_reader = BufReader::new(&stream);
                        let mut request = String::new();

                        buf_reader.read_line(&mut request).unwrap();

                        let request_line_parts: Vec<&str> = request.split_whitespace().collect();
                        if request_line_parts.len() >= 2 {
                            let method = request_line_parts[0];
                            let path_with_query = request_line_parts[1];

                            let (path, query_string) = split_path_and_query(path_with_query);

                            // Parse query parameters if present
                            let query_params: Value = match query_string {
                                Some(query_str) => queryst::parse(query_str).unwrap_or_default(),
                                None => Value::default(),
                            };

                            let mut content_length = 0;

                            loop {
                                let mut line = Default::default();
                                let res = buf_reader.read_line(&mut line);
                                if res.is_ok() {
                                    if line.starts_with("Content-Length")
                                        || line.starts_with("content-length")
                                    {
                                        let parts: Vec<_> = line.split(":").collect();
                                        content_length = parts[1].trim().parse().unwrap();
                                    }

                                    if line.as_str() == "\r\n" || line == "\n" {
                                        break;
                                    }
                                } else {
                                    break;
                                }
                            }

                            let mut body: Vec<u8> = vec![0; content_length];
                            let mut total_read: usize = 0;

                            while total_read < content_length {
                                let read_size = buf_reader.read(&mut body[total_read..]).unwrap();
                                if read_size == 0 {
                                    break;
                                }
                                total_read += read_size;
                            }

                            let req_body = match content_length {
                                0 => None,
                                _ => Some(String::from_utf8_lossy(&body).to_string()),
                            };

                            cloned_self.handle_request(
                                method,
                                path,
                                query_params,
                                stream,
                                db_pool,
                                req_body,
                            );
                        }

                        {
                            let mut active_connections = active_connections.lock().unwrap();
                            *active_connections -= 1;
                        }
                    });
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // No incoming connection, just sleep and retry
                    rb::system::sleep(rb::HZ);
                }
                Err(e) => {
                    eprintln!("Error accepting connection: {}", e);
                    break;
                }
            }

            // Check if there are no active connections (idle state)
            let active = *active_connections.lock().unwrap();
            if active == 0 {
                rb::system::sleep(rb::HZ);
            }

            // Add a small sleep to avoid tight looping when idle
            thread::sleep(Duration::from_millis(100));
        }

        Ok(())
    }

    // Handle incoming requests
    fn handle_request(
        &mut self,
        method: &str,
        path: &str,
        query_params: Value,
        mut stream: TcpStream,
        pool: sqlx::Pool<Sqlite>,
        body: Option<String>,
    ) {
        println!("{} {}", method.bright_cyan(), path);
        match self.router.route(method, path) {
            Some((handler, params)) => {
                let mut response = Response::new();
                let context = Context { pool };
                let request = Request {
                    method: method.to_string(),
                    params,
                    query_params,
                    body,
                };
                handler(&context, &request, &mut response);
                response.send(&mut stream);
            }
            None => {
                let mut response = Response::new();
                response.set_status(404);
                response.set_body("404 Not Found");
                response.send(&mut stream);
            }
        };
    }
}
