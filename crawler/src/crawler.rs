use html5ever::tokenizer::{
    Tokenizer, TokenSink, TokenSinkResult, Token, TagToken, TagKind, TokenizerOpts, BufferQueue
};
use html5ever::LocalName;
use url::Url;

struct LinkSink {
    pub base_url: Url,
    pub links: Vec<String>,
}

impl LinkSink {
    pub fn new() -> LinkSink {
        LinkSink {
            base_url: Url::parse("https://example.org").unwrap(),
            links: Vec::new(),
        } 
    }
}

impl TokenSink for LinkSink {
    type Handle = ();

    fn process_token(&mut self, token: Token, _line_number: u64) -> TokenSinkResult<Self::Handle> {
        if let TagToken(tag) = token {
            if tag.kind == TagKind::StartTag && tag.name == LocalName::from("a") {
                if let Some(href) = tag.attrs.iter().find(|&attr| attr.name.local == LocalName::from("href")) {
                    if let Ok(link) = self.base_url.join(&href.value) {
                        self.links.push(link.to_string());
                    }
                    else {
                        println!("ignored {}", href.value);
                    }
                }
            }
        }

        TokenSinkResult::Continue
    }
}

pub struct Crawler {
    frontier: Vec<String>,
}

impl Crawler {
    pub fn new(seed: Vec<String>) -> Crawler {
        Crawler {
            frontier: seed,
        }
    }

    pub async fn crawl(&mut self) {
        let mut depth = 0;
        let mut total = 0;
        let mut total_success = 0;

        while depth < 3 {
            println!("depth={}", depth);
            let mut tokenizer: Tokenizer<LinkSink> = Tokenizer::new(
                LinkSink::new(),
                TokenizerOpts::default(),
            );

            for uri in self.frontier.iter() {
                total += 1;
            
                if let Ok(base) = Url::parse(&uri) {
                    tokenizer.sink.base_url = base;
                    if let Ok(response) = reqwest::get(uri).await { 
                        if response.status() == reqwest::StatusCode::OK {
                            let headers = response.headers();
                            if headers.contains_key(reqwest::header::CONTENT_TYPE) {
                                if let Ok(content_type) = headers[reqwest::header::CONTENT_TYPE].to_str() {
                                    if let Ok(mime) = content_type.parse::<mime::Mime>() {
                                        match (mime.type_(), mime.subtype()) {
                                            (mime::TEXT, mime::HTML) => {
                                                if let Ok(body) = response.text().await {
                                                    println!("{}", uri);
                                                    total_success += 1;
                                                    let mut buffer = BufferQueue::new();
                                                    buffer.push_back(body.into());
                                                    let _ = tokenizer.feed(&mut buffer);
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

