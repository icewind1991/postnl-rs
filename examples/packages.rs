use dotenv::dotenv;
use main_error::MainError;
use postnl::PostNL;
use std::collections::HashMap;
use std::env;

#[tokio::main]
async fn main() -> Result<(), MainError> {
    dotenv().unwrap();
    let env: HashMap<_, _> = env::vars().collect();

    let client = PostNL::new()?
        .login(
            env.get("USERNAME").expect("username not set"),
            env.get("PASSWORD").expect("password not set"),
        )
        .await?;

    let packages = client.get_packages().await?;
    for package in packages.into_iter() {
        println!(
            "{}({}) - {}",
            package.generated_titles.receiver, package.key, package.delivery.status,
        );
    }
    Ok(())
}
