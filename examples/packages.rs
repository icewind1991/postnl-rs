use dotenv::dotenv;
use postnl::PostNL;
use std::collections::HashMap;
use std::env;

#[runtime::main]
async fn main() {
    dotenv().unwrap();
    let env: HashMap<_, _> = env::vars().collect();

    let client = PostNL::new(
        env.get("USERNAME").expect("username not set"),
        env.get("PASSWORD").expect("password not set"),
    );
    let packages = client.get_packages().await.unwrap();
    for package in packages.into_iter() {
        println!(
            "{}({}) - {} {}",
            package.settings.title,
            package.key,
            package.status.delivery_status,
            package
                .status
                .formatted
                .map(|status| status.short())
                .unwrap_or_default()
        );
    }
}
