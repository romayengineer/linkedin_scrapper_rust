use chromiumoxide::browser::BrowserConfigBuilder;
use chromiumoxide::{Browser, Page};
use futures::StreamExt;

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

    let pages: Vec<Page> = browser.pages().await?;

    if let Some(page) = pages.into_iter().next() {
        page.goto("https://www.google.com").await?;
    }

    println!("Browser opened! Press Enter to close...");
    std::io::stdin().read_line(&mut String::new()).ok();

    browser.close().await?;
    handle.await?;

    Ok(())
}