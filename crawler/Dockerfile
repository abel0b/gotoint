FROM rust:1.41
WORKDIR /usr/src/app
ENV RUST_BACKTRACE full
RUN cargo install cargo-watch
ENTRYPOINT ["cargo", "watch", "-x", "run crawler"]

