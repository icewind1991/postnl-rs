use dotenv::dotenv;
use main_error::MainError;
use postnl::PostNL;
use std::collections::HashMap;
use std::env;

#[tokio::main]
async fn main() -> Result<(), MainError> {
    dotenv().unwrap();
    let env: HashMap<_, _> = env::vars().collect();

    let client = PostNL::new(
        env.get("USERNAME").expect("username not set"),
        env.get("PASSWORD").expect("password not set"),
    )?;

    if let Some(token_file) = env.get("TOKENFILE") {
        match std::fs::read(token_file)
            .map_err(MainError::from)
            .and_then(|content| serde_json::from_slice(&content).map_err(MainError::from))
        {
            Ok(token) => {
                eprintln!("Restoring cached token");
                client.set_token(token)
            }
            Err(_) => {
                eprintln!("Caching token");
                let token = client.get_token().await?;
                std::fs::write(token_file, serde_json::to_vec(&token)?)?;
            }
        }
    }

    let packages = client.get_packages().await?;
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
    Ok(())
}
