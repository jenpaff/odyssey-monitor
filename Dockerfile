FROM rust:1 AS chef
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare  --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json

RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
RUN cargo build --release --bin odyssey-monitor

FROM ubuntu AS runtime

RUN apt-get update && \
  apt-get dist-upgrade -y && \
  apt-get install -y ca-certificates && \
  rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/odyssey-monitor /usr/local/bin
ENTRYPOINT ["/usr/local/bin/odyssey-monitor"]
