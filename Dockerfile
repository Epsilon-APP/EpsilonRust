FROM rust:latest

ENV RUST_BACKTRACE=full

WORKDIR /app

COPY ./ ./

RUN cargo build

RUN rm -r ./src
RUN rm Cargo.lock Cargo.toml

CMD ["./target/debug/epsilon"]