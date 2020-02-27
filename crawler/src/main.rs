mod crawler;
mod page;

use crawler::Crawler;
use std::collections::VecDeque;

#[tokio::main]
async fn main() {
    let mut seed = VecDeque::new();
    seed.push_back("https://wikipedia.org".to_string());
    seed.push_back("https://reddit.com".to_string());

    let mut crawler = Crawler::new(seed);
    crawler.crawl().await;
}
