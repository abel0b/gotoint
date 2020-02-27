use html5ever::tokenizer::{
    Tokenizer, TokenSink, TokenSinkResult, Token, CharacterTokens, TagToken, TagKind, TokenizerOpts, BufferQueue
};
use html5ever::LocalName;
use url::Url;
use crate::page::Page;
use std::collections::VecDeque;

#[derive(PartialEq, Eq)]
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
    pub fn new() -> LinkSink {
        LinkSink {
            base_url: Url::parse("https://example.org").unwrap(),
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
                                if let Some(href) = tag.attrs.iter().find(|&attr| attr.name.local == LocalName::from("href")) {
                                    if let Ok(link) = self.base_url.join(&href.value) {
                                        self.links.push_back(link.to_string());
                                    }
                                    else {
                                        println!("ignored {}", href.value);
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
                }
            },
            _ => {},
        }

        TokenSinkResult::Continue
    }
}

pub struct Crawler {
    frontier: VecDeque<String>,
}

impl Crawler {
    pub fn new(seed: VecDeque<String>) -> Crawler {
        Crawler {
            frontier: seed,
        }
    }

    pub async fn crawl(&mut self) {
        let mut depth = 0;
        let mut total = 0;
        let mut total_success = 0;

        while depth < 2 {
            println!("depth={}", depth);
            let mut tokenizer: Tokenizer<LinkSink> = Tokenizer::new(
                LinkSink::new(),
                TokenizerOpts::default(),
            );

            while let Some(url) = self.frontier.pop_front() {
                total += 1;
            
                if let Ok(base) = Url::parse(&url) {
                    tokenizer.sink.base_url = base;
                    if let Ok(response) = reqwest::get(&url).await { 
                        if response.status() == reqwest::StatusCode::OK {
                            let headers = response.headers();
                            if headers.contains_key(reqwest::header::CONTENT_TYPE) {
                                if let Ok(content_type) = headers[reqwest::header::CONTENT_TYPE].to_str() {
                                    if let Ok(mime) = content_type.parse::<mime::Mime>() {
                                        match (mime.type_(), mime.subtype()) {
                                            (mime::TEXT, mime::HTML) => {
                                                if let Ok(body) = response.text().await {
                                                                                                        total_success += 1;
                                                    let mut buffer = BufferQueue::new();
                                                    buffer.push_back(body.into());
                                                    let _ = tokenizer.feed(&mut buffer);
                                                    
                                                    println!("{}", url);
                                                    let page = Page::new(url.clone(), tokenizer.sink.extract.clone());
                                                    let client = reqwest::Client::new();
                                                    let _res = client
                                                        .post("http://admin:fixme@couchdb:5984/pages")
                                                        .json(&page)
                                                        .send()
                                                        .await.unwrap();
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
            } 
            self.frontier = tokenizer.sink.links;
            depth += 1;
        }
        println!("Successfully crawled {} pages out of {}", total_success, total);
    }
}

