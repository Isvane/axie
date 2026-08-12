FROM lukemathwalker/cargo-chef:latest-rust-1.96-alpine AS chef
WORKDIR /app
RUN apk add --no-cache musl-dev tzdata

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
RUN cargo build --release --bin axie

FROM scratch AS runtime
WORKDIR /app

COPY --from=chef /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY --from=chef /usr/share/zoneinfo /usr/share/zoneinfo

COPY ./public /app/public

COPY --from=builder /app/target/release/axie /app/axie

EXPOSE 3000

ENV RUST_LOG="info,tower_http=debug,axum::rejection=trace"

CMD ["./axie"]
