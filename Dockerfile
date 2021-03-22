FROM rust:1.50-alpine

WORKDIR app
COPY . .
RUN apk add opus opus-dev musl-dev openssl openssl-dev ffmpeg youtube-dl
RUN cargo build --release
ENTRYPOINT ["./target/release/spatula"]
