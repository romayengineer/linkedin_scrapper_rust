use chromiumoxide::browser::BrowserConfigBuilder;
use chromiumoxide::{Browser, Page};
use futures::StreamExt;

mod config;

async fn first_or_new(browser: &Browser) -> Result<Page, Box<dyn std::error::Error>> {
    let pages: Vec<Page> = browser.pages().await?;

    if let Some(page) = pages.into_iter().next() {
        page.goto("https://www.linkedin.com/login").await?;
        Ok(page)
    } else {
        let page = browser.new_page("https://www.linkedin.com/login").await?;
        Ok(page)
    }
}

async fn login(page: &Page, username: &str, password: &str) -> Result<(), Box<dyn std::error::Error>> {
    page.goto("https://www.linkedin.com/login").await?;

    let url = page.url().await?.unwrap_or_default();
    if !url.contains("/login") {
        println!("User is already logged in, current URL: {}", url);
        return Ok(());
    }

    page.find_element("input#username").await?.click().await?.type_str(username).await?;
    page.find_element("input#password").await?.click().await?.type_str(password).await?;
    page.find_element("button[type=submit]").await?.click().await?;
    Ok(())
}

async fn search_company(page: &Page) -> Result<(), Box<dyn std::error::Error>> {
    // TODO https://www.linkedin.com/search/results/companies/?keywords=aws&page=1
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    config::load();
    tracing_subscriber::fmt::init();

    let (mut browser, mut handler) = Browser::launch(
        BrowserConfigBuilder::default()
            .with_head()
            .build()?,
    )
    .await?;

    let handle = tokio::spawn(async move {
        while let Some(h) = handler.next().await {
            if h.is_err() {
                break;
            }
        }
    });

    let page = first_or_new(&browser).await?;

    login(&page, &config::username(), &config::password()).await?;

    std::io::stdin().read_line(&mut String::new()).ok();

    browser.close().await?;
    handle.await?;

    Ok(())
}