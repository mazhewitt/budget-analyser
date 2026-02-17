use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub anthropic_api_key: String,
    pub bind_address: String,
    pub database_url: String,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        // Load .env file if present (silently ignored if missing)
        let _ = dotenvy::dotenv();

        let anthropic_api_key = env::var("ANTHROPIC_API_KEY")
            .map_err(|_| "Missing ANTHROPIC_API_KEY — set it in .env or as an environment variable".to_string())?;
        let bind_address = env::var("BIND_ADDRESS").unwrap_or_else(|_| "127.0.0.1:3000".to_string());
        let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| "data/budget.db".to_string());

        Ok(Self {
            anthropic_api_key,
            bind_address,
            database_url,
        })
    }
}
