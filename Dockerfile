FROM rust:1.46 as builder
WORKDIR /usr/src/auctions_scraper
COPY . .
RUN cargo install --path .

FROM debian:buster-slim
RUN apt-get update && apt-get install -y chromium && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/auctions_scraper /usr/local/bin/auctions_scraper

CMD ["sh","-c","auctions_scraper"]
