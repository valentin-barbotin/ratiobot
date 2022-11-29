FROM rust:1.65.0 as builder


RUN USER=ratio cargo new --bin ratiobot
WORKDIR /ratiobot

COPY Cargo.* .

RUN apt update && apt install cmake -y

RUN cargo build --release
RUN rm src/*.rs

COPY ./src ./src

RUN rm ./target/release/deps/ratiobot*
RUN cargo build --release

# FROM debian:bullseye-slim as final
FROM alpine:3.17 as final

WORKDIR /app

RUN apk add --no-cache ca-certificates && rm -rf /var/cache/apk/*

COPY --from=builder /ratiobot/target/release/ratiobot .

RUN adduser -H -D svc

USER svc

CMD [ "./ratiobot" ]