use std::env;

pub fn load() {
    dotenv::dotenv().ok();
}

pub fn username() -> String {
    env::var("LINKEDIN_USERNAME").expect("LINKEDIN_USERNAME must be set")
}

pub fn password() -> String {
    env::var("LINKEDIN_PASSWORD").expect("LINKEDIN_PASSWORD must be set")
}
