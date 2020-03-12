mod crawler;
mod page;
mod webfilter;
mod urlfilter;
mod tokenizer;

use crawler::Crawler;
use std::collections::VecDeque;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let mut crawler = Crawler::new().await;
    crawler.crawl({
        let mut seed = VecDeque::new();
        seed.push_back("https://wikipedia.org".to_string());
        seed
    }).await;
}
