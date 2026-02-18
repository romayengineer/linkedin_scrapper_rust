use chromiumoxide::{Browser, Page};
use chromiumoxide::browser::BrowserConfigBuilder;
use futures::StreamExt;
use std::collections::HashSet;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinSet;
use tokio::time::{sleep, Duration};
use url::Url;

mod config;

async fn is_url_same(page: &Page, url: &str) -> Result<bool, Box<dyn Error>> {
    let page_url = page.url().await?.unwrap_or_default();
    Ok(page_url.as_str() == url)
}

async fn close_new_tabs(browser: &Browser) -> Result<(), Box<dyn Error>> {
    let pages: Vec<Page> = browser.pages().await?;

    for page in pages {
        if is_url_same(&page, "chrome://new-tab-page/").await? {
            page.close().await?;
        }
    }

    Ok(())
}

async fn element_fill(page: &Page, selector: &str, value: &str) -> Result<(), Box<dyn Error>> {
    page.find_element(selector).await?.click().await?.type_str(value).await?;
    Ok(())
}

async fn login(page: &Page, username: &str, password: &str) -> Result<(), Box<dyn Error>> {
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

async fn get_company_urls(page: &Page, urls: &mut HashSet<String>) -> Result<(), Box<dyn Error>> {
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

async fn goto_search_get_urls(page: &Page, page_index: i32) -> Result<(), Box<dyn Error>>
{
    let url = format!("https://www.linkedin.com/search/results/companies/?keywords=aws&page={}", page_index);
    page.goto(&url).await.ok();
    sleep(Duration::from_secs(2)).await;
    let mut urls: HashSet<String> = HashSet::new();
    get_company_urls(&page, &mut urls).await.ok();
    for url in urls {
        println!("page {:03} url {}", page_index, url);
    }
    Ok(())
}

async fn search_company(browser: &Browser, workers_count: i32, pages_count: i32) -> Result<(), Box<dyn Error>> {

    let mut url_rx: Vec<i32> = Vec::new();
    for i in 1..(pages_count + 1) {
        url_rx.push(i);
    }
    
    let mut page_pool: Vec<Page> = Vec::new();
    for _ in 0..workers_count {
        let page = browser.new_page("about:blank").await?;
        page_pool.push(page);
    }
    
    let url_rx = Arc::new(Mutex::new(url_rx));
    let page_pool = Arc::new(Mutex::new(page_pool));
    
    let mut workers = JoinSet::new();
    
    for _ in 0..workers_count {
        let rx = url_rx.clone();
        let pool = page_pool.clone();
        workers.spawn(async move {
            loop {
                let page_index: i32 = {
                    let mut p = rx.lock().await;
                    if p.is_empty() { break; }
                    p.pop().unwrap()
                };
                // pull page from poll
                let page: Page = pool.lock().await.pop().unwrap();
                let _ = goto_search_get_urls(&page, page_index).await;
                // push page back to poll
                pool.lock().await.push(page);
            }
        });
    }
    
    while workers.join_next().await.is_some() {}
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = config::load();
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

    login(&page, &config.username, &config.password).await?;

    search_company(&browser, config.workers, config.pages).await?;

    // wait for user press key in terminal
    // std::io::stdin().read_line(&mut String::new()).ok();

    browser.close().await?;
    handle.await?;

    // wait for browser to close
    browser.wait().await?;

    Ok(())
}