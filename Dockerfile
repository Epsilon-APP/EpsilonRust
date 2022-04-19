FROM rust:latest as build

RUN USER=root cargo new --bin epsilon
WORKDIR /epsilon

COPY ./Cargo.toml ./Cargo.toml

RUN cargo build --release
RUN rm src/*.rs

COPY ./src ./src

RUN rm ./target/release/deps/EpsilonRust*
RUN cargo build --release

#-------------------------------------------#

FROM alpine:latest

RUN apk --no-cache update && apk --no-cache add openssl-dev

WORKDIR /app

COPY --from=build /epsilon/target/release/EpsilonRust .

CMD ["ls && ./EpsilonRust"]