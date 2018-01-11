
mod exchange;

#[macro_use] extern crate serde_derive;
extern crate toml;

extern crate hmac;
extern crate sha2;
extern crate hex;

extern crate futures;
extern crate tokio_core;
#[macro_use] extern crate hyper;
extern crate hyper_tls;

extern crate serde_json;

use futures::{Future, Stream};
use tokio_core::reactor::Core;
use hyper::client::{Client};
header! { (AccessKey, "ACCESS-KEY") => [String] }
header! { (Nonce, "ACCESS-NONCE") => [u64] }
header! { (AccessSignature, "ACCESS-SIGNATURE") => [String] }
header! { (ApiSign, "apisign") => [String] }
use hyper_tls::HttpsConnector;

use serde_json::Value;

fn main() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let client = Client::configure()
        .connector(HttpsConnector::new(4, &handle).unwrap())
        .build(&handle);
        
    let access_configs = exchange::get_access_configs();   
    for access_config in access_configs {
        let req = exchange::get_req(access_config);
        if req.is_none() {
            continue;
        }
        let work = client.request(req.unwrap()).and_then(|res| {
            println!("Response: {}\n", res.status());

            res.body().concat2().and_then(move |body| {
                let v: Value = serde_json::from_slice(&body).unwrap();
                println!("{:?}", v);
                Ok(())
            })
        });
        core.run(work).unwrap();
    };
}

