# Roogle
Web search engine.

## Tasks
- [x] Crawler
    - [x] HTML parser
    - [x] Content extraction
    - [x] Database
    - [x] Visited pages bloom filter
    - [x] Multithreading
    - [x] Message queue
    - [ ] Priority queue
    - [ ] Politeness
    - [ ] Re-crawling
    - [ ] Handling crawling traps, too long urls
    - [ ] Distributed
    - [x] Language detection ([whatlang?](https://github.com/greyblake/whatlang-rs) + html tag)
    - [ ] Duplicate detection
    - [ ] DNS cache
- [ ] Index
- [ ] Query
    - [ ] Webapp

## Deploy for development
```bash
docker-compose up
```

## References
<a id="1">[1]</a>
Web Crawling
[http://infolab.stanford.edu/~olston/publications/crawling_survey.pdf](http://infolab.stanford.edu/~olston/publications/crawling_survey.pdf)

<a id="2">[2]</a>
Introduction to Information Retrieval
[https://nlp.stanford.edu/IR-book/](https://nlp.stanford.edu/IR-book/)
