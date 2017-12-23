
#[macro_use] extern crate serde_derive;
extern crate toml;

extern crate hmac;
extern crate sha2;
extern crate hex;

extern crate futures;
extern crate tokio_core;
#[macro_use] extern crate hyper;
extern crate hyper_tls;

use std::fs::File;
use std::io::{self, Write};
use std::io::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};

use sha2::Sha256;
use hmac::{Hmac, Mac};
use hex::ToHex;

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

    let access_config = get_access_config("coincheck").unwrap();
    let uri = format!("https://{}{}", access_config.host, "/api/accounts/balance");
    let body = "";
    let access_key = access_config.access_key;
    let secret_key = access_config.secret_key;

    let access_nonce = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

    let msg = format!("{}{}{}", access_nonce.to_string().as_str(), uri, body);
    let mut hmac = Hmac::<Sha256>::new(secret_key.as_bytes()).unwrap();
    hmac.input(msg.as_bytes());
    let mut access_signature = String::new();
    hmac.result().code().write_hex(&mut access_signature).unwrap();

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

#[derive(Deserialize)]
struct Config {
    access_configs: Vec<AccessConfig>,
}

#[derive(Deserialize)]
struct AccessConfig {
    id: String,
    host: String,
    access_key: String,
    secret_key: String,
}

fn get_access_config(id: &str) -> Option<AccessConfig> {

    let mut file = File::open("Shalltear.toml").expect("File not found");
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("Something went wrong reading the file");

    let config: Config = toml::from_str(contents.as_str()).expect("Failed to create toml string");
    config.access_configs.into_iter().find(|x| x.id == id)
}
