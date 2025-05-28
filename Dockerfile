FROM rust:latest AS builder

RUN apt-get update && apt-get install -y pkg-config libssl-dev

WORKDIR /app
COPY . .

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && rm -rf /var/lib/apt/lists/*

RUN groupadd -r -g 1000 docker_group && \
    useradd --no-log-init -r -u 1000 -g docker_group docker_user

USER docker_user
WORKDIR /home/docker_user

COPY --chown=docker_user:docker_group --chmod=500 --from=builder /app/target/release/event-tracker /usr/local/bin/event-tracker
COPY --chown=docker_user:docker_group --chmod=600 log4rs.yml .

EXPOSE 8080

CMD ["event-tracker"]
