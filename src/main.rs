use chromiumoxide::browser::BrowserConfigBuilder;
use chromiumoxide::{Browser, Page};
use futures::StreamExt;

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    let _page = first_or_new(&browser).await?;

    std::io::stdin().read_line(&mut String::new()).ok();

    browser.close().await?;
    handle.await?;

    Ok(())
}