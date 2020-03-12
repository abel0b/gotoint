use html5ever::tokenizer::{
    Tokenizer, TokenSink, TokenSinkResult, Token, CharacterTokens, TagToken, TagKind, TokenizerOpts, BufferQueue
};
use html5ever::LocalName;
use url::Url;
use crate::page::Page;
use std::collections::VecDeque;
use std::net::ToSocketAddrs;
use lapin::{
    message::DeliveryResult, options::*, types::FieldTable, ConsumerDelegate,
};
use std::sync::{Mutex, Arc};
use log::{info, trace};
use crate::webfilter;
use crate::urlfilter;
use crate::tokenizer;

#[derive(Eq, PartialEq)]
enum LinkSinkState {
    Start,
    Body,
    Script,
    Style,
}

struct LinkSink {
    pub base_url: Url,
    pub links: VecDeque<String>,
    pub extract: String,
    pub state: LinkSinkState,
}

impl LinkSink {
    pub fn new(base_url: Url) -> LinkSink {
        LinkSink {
            base_url,
            links: VecDeque::new(),
            extract: String::new(),
            state: LinkSinkState::Start,
        } 
    }
}

impl TokenSink for LinkSink {
    type Handle = ();

    fn process_token(&mut self, token: Token, _line_number: u64) -> TokenSinkResult<Self::Handle> {
        if self.state == LinkSinkState::Start {
            if let TagToken(tag) = token {
                if tag.name == LocalName::from("body") {
                    self.state = LinkSinkState::Body;
                }
            }
            return TokenSinkResult::Continue;
        }
        match token {
            TagToken(tag) => {
                match tag.kind {
                    TagKind::StartTag => {
                        if self.state == LinkSinkState::Script || self.state == LinkSinkState::Style {
                            self.state = LinkSinkState::Body;     
                        }

                       if tag.name == LocalName::from("a") {
                           // TODO: ignore nofollow links
                            if let Some(href) = tag.attrs.iter().find(|&attr| attr.name.local == LocalName::from("href")) {
                                if let Ok(link) = self.base_url.join(&href.value) {
                                    let url = link.to_string();
                                    if urlfilter::pass(&url) {
                                        self.links.push_back(url);
                                    }
                                }
                            }
                        }
                        else if tag.name == LocalName::from("script") {
                            self.state = LinkSinkState::Script; 
                        }
                        else if tag.name == LocalName::from("style") {
                            self.state = LinkSinkState::Style;
                        }
                    },
                    TagKind::EndTag => {
                        if tag.name == LocalName::from("script") || tag.name == LocalName::from("style") {
                            self.state = LinkSinkState::Body;
                        }
                    },
                }
            },
            CharacterTokens(str) => {
                if self.state == LinkSinkState::Body {
                    self.extract.push_str(&str);
                    self.extract.push(' ');
                }
            },
            _ => {},
        }

        TokenSinkResult::Continue
    }
}

struct Worker {
    id: u32,
    http_client: reqwest::Client,
    crawl_jobs: Arc<Mutex<VecDeque<String>>>,
    redis: redis::aio::MultiplexedConnection,
    crawl_pub_chan: lapin::Channel,
    total: u64,
    total_success: u64,
}

impl Worker {
    const VISITED_KEY: &'static str = "visited_pages";

    pub fn new(id: u32, crawl_jobs: Arc<Mutex<VecDeque<String>>>, redis: redis::aio::MultiplexedConnection, crawl_pub_chan: lapin::Channel) -> Worker {
        let http_client = reqwest::ClientBuilder::new()
            .default_headers({
                let mut headers = reqwest::header::HeaderMap::new();
                headers.insert(reqwest::header::USER_AGENT, reqwest::header::HeaderValue::from_str(format!("gotoint/{} Web search engine {}", env!("CARGO_PKG_VERSION"), env!("CARGO_PKG_HOMEPAGE")).as_str()).unwrap());
                headers
            })
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .unwrap();
        Worker {
            id,
            http_client,
            crawl_jobs,
            total: 0,
            total_success: 0,
            redis,
            crawl_pub_chan,
        }
    }

    pub async fn run(&mut self) {
        trace!("worker {} > started", self.id);
        loop {
            let url = {
                let next_job = {
                    self.crawl_jobs.lock().unwrap().pop_front()
                };
                match next_job {
                    Some(url) => url,
                    None => {
                        trace!("worker {} > continue", self.id);
                        tokio::time::delay_for(
                            std::time::Duration::from_secs(10)
                        ).await;
                        continue
                    },
                }
            };

            self.total += 1;
            trace!("worker {} > fetch {}", self.id, url);

            let mut newlinks: Option<VecDeque<String>> = None;
            
            if let Ok(base) = Url::parse(&url) {
                if let Ok(response) = self.http_client.get(&url).send().await {
                    trace!("worker {} > got {}", self.id, url);
                    if response.status() == reqwest::StatusCode::OK {
                        let headers = response.headers();
                        if headers.contains_key(reqwest::header::CONTENT_TYPE) {
                            if let Ok(content_type) = headers[reqwest::header::CONTENT_TYPE].to_str() {
                                if let Ok(mime) = content_type.parse::<mime::Mime>() {
                                    match (mime.type_(), mime.subtype()) {
                                        (mime::TEXT, mime::HTML) => {
                                            if let Ok(body) = response.text().await {
                                                trace!("worker {} > parse {}", self.id, url);
                                                let (links, extract) = {
                                                    let mut html_tokenizer: Tokenizer<LinkSink> = Tokenizer::new(
                                                        LinkSink::new(base),
                                                        TokenizerOpts::default(),
                                                    );

                                                    let mut buffer = BufferQueue::new();
                                                    buffer.push_back(body.into());
                                                    let _ = html_tokenizer.feed(&mut buffer);
                                                    (html_tokenizer.sink.links, tokenizer::process(html_tokenizer.sink.extract))
                                                };
                                                
                                                let res: redis::RedisResult<bool> = redis::cmd("BF.ADD").arg(Self::VISITED_KEY).arg(&url).query_async(&mut self.redis).await;
                                                res.unwrap();

                                                let page = Page::new(url.clone(), extract);
                                                if webfilter::pass(&page) {
                                                    self.total_success += 1;
                                                    newlinks = Some(links);
                                                    let _res = self.http_client
                                                    .post("http://admin:fixme@couchdb:5984/pages")
                                                    .json(&page)
                                                    .send()
                                                    .await.unwrap();
                                                    trace!("worker {} > save {}", self.id, url);
                                                }
                                                else {
                                                    trace!("worker {} > throw {}", self.id, url);
                                                }
                                            }
                                        },
                                        _ => {},
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if let Some(links) = newlinks {
                trace!("worker {} > add links {}", self.id, url);
                // TODO: join
                for newurl in links {
                    let res: redis::RedisResult<bool> = redis::cmd("BF.EXISTS").arg(Self::VISITED_KEY).arg(&newurl).query_async(&mut self.redis).await;
                    let maybe_visited = res.unwrap();
                    if !maybe_visited {
                        self.crawl_pub_chan
                        .basic_publish(
                            "",
                            "crawl_jobs",
                            lapin::options::BasicPublishOptions::default(),
                            newurl.into_bytes(),
                            lapin::BasicProperties::default(),
                        )
                        .await
                        .unwrap();
                    }
                }
                trace!("worker {} > added links {}", self.id, url);
            }
            info!("total {}/{}", self.total_success, self.total);
        }
    }
}

struct JobListener {
    channel: lapin::Channel,
    crawl_jobs: Arc<Mutex<VecDeque<String>>>,
}

impl JobListener {
    pub fn new(channel: lapin::Channel, crawl_jobs: Arc<Mutex<VecDeque<String>>>) -> JobListener {
        JobListener {
            channel,
            crawl_jobs,
        }
    }
}

impl ConsumerDelegate for JobListener {
    fn on_new_delivery(&self, delivery: DeliveryResult) {
        if let Ok(Some(delivery)) = delivery {
            self.channel
                .basic_ack(delivery.delivery_tag, BasicAckOptions::default())
                .wait()
                .expect("basic_ack");
            let mut crawl_jobs = self.crawl_jobs.lock().unwrap();
            crawl_jobs.push_back(
                String::from_utf8(delivery.data).unwrap()
            );
        }
    }
}

pub struct Crawler {
    redis: redis::aio::MultiplexedConnection,
    crawl_jobs: Arc<Mutex<VecDeque<String>>>,
    rabbit_con: lapin::Connection,
    crawl_pub_chan: lapin::Channel,
    crawl_cons_chan: lapin::Channel,
}

impl ConsumerDelegate for Crawler {
    fn on_new_delivery(&self, delivery: DeliveryResult) {
        if let Ok(Some(delivery)) = delivery {
            self.crawl_cons_chan
                .basic_ack(delivery.delivery_tag, BasicAckOptions::default())
                .wait()
                .expect("basic_ack");
         }
    }
}

impl Crawler {
    const CAPACITY: u32 = 100000;
    const ERROR_RATE: f32 = 0.001;
    const VISITED_KEY: &'static str = "visited_pages";

    pub async fn new() -> Crawler { 
        let redis_client = redis::Client::open("redis://redis").unwrap(); 
        let redis = redis_client.get_multiplexed_tokio_connection().await.unwrap();
        
        let mut rabbit_addr = "rabbitmq:5672".to_socket_addrs().unwrap();
        let rabbit_addr = format!("amqp://{}/%2f", rabbit_addr.next().unwrap().to_string());
        let rabbit_con = lapin::Connection::connect(&rabbit_addr, lapin::ConnectionProperties::default()).await.unwrap();
        let crawl_pub_chan = rabbit_con.create_channel().await.unwrap();
        let crawl_cons_chan = rabbit_con.create_channel().await.unwrap();
        Crawler {
            crawl_jobs: Arc::new(Mutex::new(VecDeque::new())),
            redis,
            rabbit_con,
            crawl_pub_chan,
            crawl_cons_chan,
        }
    }

    pub async fn crawl(&mut self, mut seed: VecDeque<String>) {
        let res : redis::RedisResult<bool> = redis::Cmd::exists(Self::VISITED_KEY).query_async(&mut self.redis).await;
        if res.unwrap() {
            let res : redis::RedisResult<bool> = redis::Cmd::del(Self::VISITED_KEY).query_async(&mut self.redis).await;
            res.unwrap();
        }

        let res: redis::RedisResult<String> = redis::cmd("BF.RESERVE").arg(Self::VISITED_KEY).arg(Self::ERROR_RATE).arg(Self::CAPACITY).query_async(&mut self.redis).await;
        res.unwrap();

        let http_client = reqwest::Client::new();
        http_client
            .put("http://admin:fixme@couchdb:5984/pages?n=1")
            .send()
            .await.unwrap();

        let _queue = self.crawl_pub_chan
            .queue_declare(
                "crawl_jobs",
                lapin::options::QueueDeclareOptions::default(),
                lapin::types::FieldTable::default(),
            )
            .await
            .unwrap();
    
        // TODO: join
        while let Some(url) = seed.pop_front() { 
            self.crawl_pub_chan
                .basic_publish(
                    "",
                    "crawl_jobs",
                    lapin::options::BasicPublishOptions::default(),
                    url.into_bytes(),
                    lapin::BasicProperties::default(),
                )
                .await
                .unwrap();
        }

        let ch2 = self.crawl_cons_chan.clone();
        self.crawl_cons_chan
            .basic_consume(
                "crawl_jobs",
                "crawler",
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await
            .unwrap()
            .set_delegate(Box::new(JobListener::new(ch2, Arc::clone(&self.crawl_jobs))));

        let num_workers: u32 = 6;
        let mut workers = Vec::new();
        info!("starting workers");
        for worker_id in 1..=num_workers {
            let crawl_jobs = Arc::clone(&self.crawl_jobs);
            let redis = self.redis.clone();
            let crawl_pub_chan = self.crawl_pub_chan.clone();

            workers.push(
                tokio::spawn(async move {
                    let mut worker = Worker::new(worker_id, crawl_jobs, redis, crawl_pub_chan);
                    worker.run().await;
                })
            );
        }

        for worker in workers {
            worker.await.unwrap();
        }

        self.rabbit_con.close(200, "OK").await.unwrap();
    }
}

