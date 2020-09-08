FROM rust:1.46-slim-buster

VOLUME [ "/build" ]
VOLUME [ "/output" ]
VOLUME [ "/target" ]

WORKDIR /build
COPY . .

ENTRYPOINT [ "/bin/sh", "Dockerscript" ]
