FROM rust:latest as build

WORKDIR /rust-build

COPY ./ ./

RUN cargo build

# ------------------------------------------------------------------------------
# Stage
# ------------------------------------------------------------------------------

FROM alpine:latest

WORKDIR /app

COPY --from=build rust-build/target/debug/EpsilonRust .

CMD ["./EpsilonRust"]