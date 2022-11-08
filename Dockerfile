FROM rust:1.64.0 as builder

RUN update-ca-certificates

WORKDIR /app
COPY . .

RUN cargo build --release

FROM debian:buster-slim

WORKDIR /app
COPY --from=builder /app/target/release/chaosbot ./
COPY --from=builder /app/assets/ ./

CMD ["./chaosbot"]