FROM rust:latest

WORKDIR /app

COPY ./ ./

RUN cargo build

RUN rm -r ./src
RUN rm Cargo.lock Cargo.toml

CMD ["./target/debug/EpsilonRust"]