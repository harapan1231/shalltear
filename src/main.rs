
extern crate crypto;
extern crate rustc_serialize;

extern crate futures;
extern crate tokio_core;
#[macro_use] extern crate hyper;
extern crate hyper_tls;

use std::io::{self, Write};
use std::time::{SystemTime, UNIX_EPOCH};
use crypto::sha2::Sha256;
use crypto::hmac::Hmac;
use crypto::mac::Mac;
use rustc_serialize::hex::ToHex;

use futures::{Future, Stream};
use tokio_core::reactor::Core;
use hyper::Method;
use hyper::client::{Client, Request};
header! { (AccessKey, "ACCESS-KEY") => [String] }
header! { (AccessNonce, "ACCESS-NONCE") => [u64] }
header! { (AccessSignature, "ACCESS-SIGNATURE") => [String] }
use hyper_tls::HttpsConnector;

fn main() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let client = Client::configure()
        .connector(HttpsConnector::new(4, &handle).unwrap())
        .build(&handle);

    let uri = "***";
    let body = "***";
    let access_key = "***";
    let secret_key = "***";

    let access_nonce = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

    let msg = format!("{}{}{}", access_nonce.to_string().as_str(), uri, body);
    let mut hmac = Hmac::new(Sha256::new(), secret_key.as_bytes());
    hmac.input(msg.as_bytes());
    let access_signature = hmac.result().code().to_hex();
    println!("{}", access_signature);

    let mut req = Request::new(Method::Get, uri.parse().unwrap());
    req.headers_mut().set(AccessKey(access_key.to_string()));
    req.headers_mut().set(AccessNonce(access_nonce));
    req.headers_mut().set(AccessSignature(access_signature));

    let work = client.request(req).and_then(|res| {
        println!("Response: {}", res.status());

        res.body().for_each(|chunk| {
            io::stdout()
                .write_all(&chunk)
                .map_err(From::from)
        })
    });
    core.run(work).unwrap();
}

