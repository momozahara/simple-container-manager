FROM rust:1.70 as builder
WORKDIR /app

COPY . .
RUN cargo install --path .

FROM debian:bullseye-slim

ENV CONTAINER_NAME container
ENV HOST 127.0.0.1
ENV PORT 2375

RUN apt-get update && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/containers-manager /usr/local/bin/containers-manager
COPY --from=builder /app/static /usr/share/myapp/static

CMD containers-manager --name ${CONTAINER_NAME} --host ${HOST} --port ${PORT} --path /usr/share/myapp/static/
