FROM rust:1.59

WORKDIR /usr/src/json_validator

COPY . .

RUN cargo install --path .

COPY ./entrypoint /entrypoint

ENTRYPOINT ["/entrypoint"]
