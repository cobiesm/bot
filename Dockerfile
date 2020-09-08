FROM rust:1.46-slim-buster

VOLUME [ "/build" ]
VOLUME [ "/output" ]
VOLUME [ "/target" ]

WORKDIR /build

ENTRYPOINT [ "/bin/sh", "Dockerscript" ]
