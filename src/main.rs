
extern crate futures;
extern crate tokio_core;
extern crate hyper;

use std::io::{self, Write};
use futures::{Future, Stream};
use tokio_core::reactor::Core;
use hyper::Client;

fn main() {
    let mut core = Core::new().unwrap();
    let client = Client::new(&core.handle());

    let uri = "http://httpbin.org/ip".parse().unwrap();
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
