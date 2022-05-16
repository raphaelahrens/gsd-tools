FROM rust:1.59

WORKDIR /usr/src/json_validator

COPY ./Cargo.toml ./Cargo.toml
COPY ./src ./src

RUN cargo install --path .

COPY ./entrypoint /entrypoint

ENTRYPOINT ["/entrypoint"]
