mod auth;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Rustified Minecraft Launcher");
    println!("---------------------------");
    
    // Initialize the authentication process
    println!("Starting authentication process...");
    let auth_result = auth::authenticate().await?;
    println!("Authentication successful!");
    println!("Welcome, {}!", auth_result.profile.name);
    
    // Here you would continue with the launcher functionality
    
    Ok(())
}
