
pub mod bittrex;
pub mod coincheck;


extern crate toml;

extern crate hmac;
extern crate sha2;
extern crate hex;

extern crate futures;
extern crate tokio_core;
extern crate hyper_tls;

use std::fs::File;
use std::io::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};

use sha2::{Sha256, Sha512};
use hmac::{Hmac, Mac};
use hex::ToHex;

use hyper::Method;
use hyper::client::Request;
header! { (AccessKey, "ACCESS-KEY") => [String] }
header! { (Nonce, "ACCESS-NONCE") => [u64] }
header! { (AccessSignature, "ACCESS-SIGNATURE") => [String] }
header! { (ApiSign, "apisign") => [String] }

#[derive(Deserialize)]
struct Config {
    access_configs: Vec<AccessConfig>,
}

#[derive(Deserialize)]
pub struct AccessConfig {
    service_id: String,
    api_key: String,
    secret_key: String,
}

pub fn get_access_configs() -> Vec<AccessConfig> {

    let mut file = File::open("Shalltear.toml").expect("File not found");
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("Something went wrong reading the file");

    let config: Config = toml::from_str(contents.as_str()).expect("Failed to create toml string");
    return config.access_configs
}

pub fn get_req(access_config: AccessConfig) -> Option<Request> {

    let service_id = access_config.service_id.as_str();
    let api_key = access_config.api_key;
    let secret_key = access_config.secret_key;

    if service_id.is_empty() || api_key.is_empty() || secret_key.is_empty() {
        println!("Skip because of insufficient params...\n[\"{}\"]\n", service_id);
        return None
    }
    println!("Starts to connect...\n[\"{}\"]\n", service_id);

    let nonce = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

    let url = get_url(service_id, api_key.as_str(), nonce.to_string().as_str());

    let sign_msg = match service_id {
        "bittrex" => url.clone(),
        "coincheck" => format!("{}{}{}", nonce.to_string().as_str(), url, ""),
        _ => String::new(),
    };

    let sign = get_sign(service_id, secret_key.as_bytes(), sign_msg.as_str());

    let mut req = Request::new(Method::Get, url.parse().unwrap());

    match service_id {
        "bittrex" => {
            req.headers_mut().set(ApiSign(sign));
        }
        "coincheck" => {
            req.headers_mut().set(AccessKey(api_key.to_string()));
            req.headers_mut().set(Nonce(nonce));
            req.headers_mut().set(AccessSignature(sign.clone()));
        }
        _ => { }
    };

    return Some(req)
}

fn get_url(service_id: &str, api_key: &str, nonce: &str) -> String {

    let mut ret: String = String::new();
    
    ret.push_str("https://");
    let url_body = match service_id {
        "bittrex" => format!("{}?apikey={}&nonce={}", 
            "bittrex.com/api/v1.1/account/getbalances",
            api_key,
            nonce,
        ),
        "coincheck" => "coincheck.com/api/accounts/balance".to_string(),
        _ => String::new(),
    };
    ret.push_str(url_body.as_str());

    return ret
}

fn get_sign(service_id: &str, secret_key: &[u8], sign_msg: &str) -> String {

    let mut ret = String::new();

    match service_id {
        "bittrex" => {
            let mut hmac = Hmac::<Sha512>::new(secret_key).unwrap();
            hmac.input(sign_msg.as_bytes());
            hmac.result().code().write_hex(&mut ret).unwrap()
        }
        "coincheck" => {
            let mut hmac = Hmac::<Sha256>::new(secret_key).unwrap();
            hmac.input(sign_msg.as_bytes());
            hmac.result().code().write_hex(&mut ret).unwrap()
        }
        _ => { }
    };

    return ret
}

