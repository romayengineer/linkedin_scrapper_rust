use chromiumoxide::browser::BrowserConfigBuilder;
use chromiumoxide::{Browser, Page};
use futures::StreamExt;
use tokio::time::{sleep, Duration};
use url::Url;
use std::collections::HashSet;

mod config;

async fn is_url_same(page: &Page, url: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let page_url = page.url().await?.unwrap_or_default();
    Ok(page_url.as_str() == url)
}

async fn close_new_tabs(browser: &Browser) -> Result<(), Box<dyn std::error::Error>> {
    let pages: Vec<Page> = browser.pages().await?;

    for page in pages {
        if is_url_same(&page, "chrome://new-tab-page/").await? {
            page.close().await?;
        }
    }

    Ok(())
}

async fn element_fill(page: &Page, selector: &str, value: &str) -> Result<(), Box<dyn std::error::Error>> {
    page.find_element(selector).await?.click().await?.type_str(value).await?;
    Ok(())
}

async fn login(page: &Page, username: &str, password: &str) -> Result<(), Box<dyn std::error::Error>> {
    if is_url_same(&page, "https://www.linkedin.com/feed/").await? {
        println!("User is already logged in");
        return Ok(())
    }

    page.goto("https://www.linkedin.com/login").await?;

    if is_url_same(&page, "https://www.linkedin.com/feed/").await? {
        println!("User is already logged in");
        return Ok(())
    }

    element_fill(&page, "input#username", &username).await?;
    element_fill(&page, "input#password", &password).await?;
    page.find_element("button[type=submit]").await?.click().await?;
    Ok(())
}

async fn get_company_urls(page: &Page, urls: &mut HashSet<String>) -> Result<(), Box<dyn std::error::Error>> {
    let links = page.find_elements("a").await?;
    for link in links {
        if let Ok(Some(href)) = link.attribute("href").await {
            if href.starts_with("https://www.linkedin.com/company/") {
                let parsed = Url::parse(&href)?;
                let path = parsed.path();
                let first_two = format!("/{}", path.split('/').skip(1).take(2).collect::<Vec<&str>>().join("/"));
                let clean_url = format!("{}://{}{}", parsed.scheme(), parsed.host_str().unwrap_or_default(), first_two);
                urls.insert(clean_url.clone());
            }
        }
    }
    Ok(())
}

async fn search_company(page: &Page) -> Result<(), Box<dyn std::error::Error>> {
    for i in 1..11 {
        let url = format!("https://www.linkedin.com/search/results/companies/?keywords=aws&page={:?}", i);
        page.goto(url).await?;
        sleep(Duration::from_secs(2)).await;
        let mut urls: HashSet<String> = HashSet::new();
        get_company_urls(&page, &mut urls).await?;
        for url in urls {
            println!("page {:?} url {:?}", i, url);
        }
    }
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

    let page = browser.new_page("https://www.linkedin.com").await?;

    close_new_tabs(&browser).await?;

    login(&page, &config::username(), &config::password()).await?;

    search_company(&page).await?;

    std::io::stdin().read_line(&mut String::new()).ok();

    browser.close().await?;
    handle.await?;

    Ok(())
}