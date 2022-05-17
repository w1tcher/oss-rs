
use oss::auth;
use oss::client;
use oss::auth::VERB;

extern crate dotenv;

use dotenv::dotenv;
use std::env;

fn main() {
    dotenv().ok();

    let key_id      = env::var("ALIYUN_KEY_ID").unwrap().to_owned();
    let key_secret  = env::var("ALIYUN_KEY_SECRET").unwrap();
    let endpoint    = env::var("ALIYUN_ENDPOINT").unwrap();
    let bucket      = env::var("ALIYUN_BUCKET").unwrap();

    let client = client::Client::new(&key_id,&key_secret, &endpoint, &bucket);
    //let headers = None;
    let response = client.get_bucket_list().unwrap();
    println!("{:?}", response);
}
