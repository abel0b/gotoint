mod crawler;

use crawler::Crawler;

#[tokio::main]
async fn main() {
    let seed = vec![
        "http://en.wikipedia.org/wiki/Internet".to_string(),
        //"http://reddit.com/".to_string(),
        //"http://github.com/".to_string(),
    ];

    let mut crawler = Crawler::new(seed);
    crawler.crawl().await;
}
