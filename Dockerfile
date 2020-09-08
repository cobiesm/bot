FROM rust:1.46-slim-buster

VOLUME [ "/build" ]
VOLUME [ "/output" ]
VOLUME [ "/target" ]
VOLUME [ "/usr/local/cargo" ]

WORKDIR /build

ENTRYPOINT [ "/bin/sh", "Dockerscript" ]
