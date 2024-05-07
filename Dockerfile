FROM rust:alpine as base

RUN apk add --no-cache musl-dev
RUN cargo install cargo-chef 
COPY . .

FROM base as prepare 

COPY . .
RUN cargo chef prepare --recipe-path recipe.json 

FROM base as build 
COPY --from=prepare recipe.json . 
RUN cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json
COPY . . 
RUN cargo build --release --target x86_64-unknown-linux-musl --bin schmfy_express
RUN strip /target/x86_64-unknown-linux-musl/release/schmfy_express

FROM alpine:latest as run
COPY --from=build /target/x86_64-unknown-linux-musl/release/schmfy_express .

ENTRYPOINT [ "./schmfy_express" ]
