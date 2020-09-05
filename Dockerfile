FROM rust:1.46-slim-buster

WORKDIR /build
COPY . .

VOLUME [ "/output" ]
ENTRYPOINT [ "/bin/sh", "Dockerscript" ]
