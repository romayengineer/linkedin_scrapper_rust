use std::env;

pub struct Config {
    pub username: String,
    pub password: String,
    pub workers: i32,
    pub pages: i32,
}

pub fn load() -> Config {
    dotenv::dotenv().ok();
    Config {
        username: env::var("USERNAME")
            .expect("USERNAME must be set"),
        password: env::var("PASSWORD")
            .expect("PASSWORD must be set"),
        workers: env::var("WORKERS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(4),
        pages: env::var("PAGES")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(20)
    }
}
