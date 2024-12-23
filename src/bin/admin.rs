use rust_scraper::api::{
    middleware::auth::{AuthMiddleware, AuthConfig, Role},
    models::auth::RegisterRequest,
};
use std::io::{self, Write};

fn read_input(prompt: &str) -> io::Result<String> {
    print!("{}", prompt);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Initial Admin Setup");
    println!("------------------");

    let email = read_input("Enter admin email: ")?;
    let password = read_input("Enter admin password: ")?;
    let name = read_input("Enter admin name: ")?;

    let register_req = RegisterRequest {
        email: email.clone(),
        password,
        name,
        role: Some(Role::Admin),
    };

    let config = AuthConfig {
        jwt_secret: "your-secret-key".to_string(), // In production, this should be loaded from environment
        token_expiration: 24 * 60 * 60, // 24 hours
    };

    let auth = AuthMiddleware::new(config);
    let token = auth.generate_token(&register_req.email, Role::Admin)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    println!("\nAdmin user created successfully!");
    println!("JWT Token: {}", token);
    println!("\nIMPORTANT: Store this token securely. It will be required for creating other users.");

    Ok(())
} 