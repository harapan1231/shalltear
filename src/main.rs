
extern crate futures;
extern crate tokio_core;
extern crate hyper;
extern crate hyper_tls;

use std::io::{self, Write};

use futures::{Future, Stream};

use tokio_core::reactor::Core;

use hyper::Client;
use hyper_tls::HttpsConnector;

fn main() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let client = Client::configure()
        .connector(HttpsConnector::new(4, &handle).unwrap())
        .build(&handle);

    let uri = "https://httpbin.org/ip".parse().unwrap();

    let work = client.get(uri).and_then(|res| {
        println!("Response: {}", res.status());

        res.body().for_each(|chunk| {
            io::stdout()
                .write_all(&chunk)
                .map_err(From::from)
        })
    });
    core.run(work).unwrap();
}

