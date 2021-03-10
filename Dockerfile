FROM rust:1.50

WORKDIR app
COPY . .
RUN cargo build --release
ENTRYPOINT ["./target/release/spatula"]
