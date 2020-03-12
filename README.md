# gotoint
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
    - [x] Language detection
    - [ ] Duplicate detection
    - [ ] DNS cache
- [ ] Index
- [ ] Query
    - [ ] Webapp
- [ ] Project name

Check out
- [SMOG](https://en.wikipedia.org/wiki/SMOG)
- [Bloom filter](https://en.wikipedia.org/wiki/Bloom_filter)
- [TrustRank](https://en.wikipedia.org/wiki/TrustRank)
- [HITS algorithm](https://en.wikipedia.org/wiki/HITS_algorithm)

## Deploy for development
Crawl pages.
```bash
docker-compose -f deploy/crawler.dev.yml up
```
Build inverted index.
```bash
docker-compose -f deploy/index.dev.yml up
```
Start web server.
```bash
docker-compose -f deploy/dev.yml up
```

## References
<a id="1">[1]</a>
Web Crawling
[http://infolab.stanford.edu/~olston/publications/crawling_survey.pdf](http://infolab.stanford.edu/~olston/publications/crawling_survey.pdf)

<a id="2">[2]</a>
Introduction to Information Retrieval
[https://nlp.stanford.edu/IR-book/](https://nlp.stanford.edu/IR-book/)
