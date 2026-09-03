use std::env;

pub struct Config {
    pub host: String,
    pub port: u16,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        let host = env::var("CELEBRIMBOR_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());

        let port = env::var("CELEBRIMBOR_PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse::<u16>()
            .map_err(|error| format!("invalid CELEBRIMBOR_PORT: {error}"))?;

        Ok(Self { host, port })
    }
}
