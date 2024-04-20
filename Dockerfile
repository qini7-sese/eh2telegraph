FROM rust:1-bullseye as builder
WORKDIR /usr/src/eh2telegraph
COPY . .
RUN cargo update
RUN cargo build --release

FROM debian:bullseye-slim
RUN apt-get update && apt-get -y install ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/eh2telegraph/target/release/bot /usr/local/bin/bot
CMD ["/usr/local/bin/bot"]