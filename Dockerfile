FROM rust:latest

WORKDIR /app

COPY ./ ./

RUN cargo build

RUN rm -r ./src Cargo.lock Cargo.toml

CMD ["./target/debug/EpsilonRust"]