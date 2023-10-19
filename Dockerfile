FROM rust:1.71.0-slim as builder

RUN apt-get update \
 && DEBIAN_FRONTEND=noninteractive \
    apt-get install --no-install-recommends --assume-yes \
      protobuf-compiler

RUN mkdir /usr/src/unit
WORKDIR /usr/src/unit

COPY . .

RUN cargo build --release

RUN ls -la ./target/release 

FROM rust:1.71.0-slim 
COPY --from=builder /usr/src/unit/target/release/unit-node /usr/local/bin
COPY --from=builder /usr/src/unit/target/release/unit-api /usr/local/bin

