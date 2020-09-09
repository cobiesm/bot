FROM rust:1.46-slim-buster

VOLUME [ "/build" ]
VOLUME [ "/output" ]
VOLUME [ "/target" ]
VOLUME [ "/usr/local/cargo" ]

WORKDIR /build

RUN apt-get update && apt-get install -y --no-install-recommends libssl-dev pkg-config

ENTRYPOINT [ "/bin/sh", "Dockerscript" ]
