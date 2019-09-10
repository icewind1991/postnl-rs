use dotenv::dotenv;
use postnl::PostNL;
use std::collections::HashMap;
use std::env;

fn main() {
    dotenv().unwrap();
    let env: HashMap<_, _> = env::vars().collect();

    let client = PostNL::new(
        env.get("USERNAME").expect("username not set"),
        env.get("PASSWORD").expect("password not set"),
    )
    .unwrap();
    for package in client.get_packages().unwrap().into_iter() {
        println!(
            "{}({}) - {}",
            package.settings.title, package.key, package.status.delivery_status
        );
    }
}
