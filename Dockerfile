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

FROM debian:bullseye-slim as final

WORKDIR /app

RUN apt-get update -y && apt install ca-certificates -y && apt-get clean

COPY --from=builder /ratiobot/target/release/ratiobot .

RUN useradd -M -s /bin/bash -u 1001 svc

USER svc

CMD [ "./ratiobot" ]