FROM rust:latest as build

RUN apt-get update

RUN apt-get install musl-tools -y

RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /rust-build

COPY ./ ./

RUN RUSTFLAGS=-Clinker=musl-gcc cargo build --target=x86_64-unknown-linux-musl
RUN rm -f target/x86_64-unknown-linux-musl/debug/deps/EpsilonRust*

# ------------------------------------------------------------------------------
# Stage
# ------------------------------------------------------------------------------

FROM alpine:latest

WORKDIR /app

COPY --from=build /rust-build/target/x86_64-unknown-linux-musl/debug/EpsilonRust .

CMD ["./EpsilonRust"]