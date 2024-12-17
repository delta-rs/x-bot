FROM rust:alpine as chef
ARG ARCH
ENV SYSROOT /dummy

WORKDIR /build
RUN apk add musl-dev
RUN cargo install cargo-chef --locked
RUN rustup toolchain install 1.83.0-${ARCH}-unknown-linux-musl

FROM chef as planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder
COPY --from=planner /build/recipe.json recipe.json
RUN apk add libressl-dev musl-dev
RUN cargo chef cook --release --target=${ARCH}-unknown-linux-musl --recipe-path recipe.json
COPY . .
RUN cargo build --release --target=${ARCH}-unknown-linux-musl
RUN mv /build/target/${ARCH}-unknown-linux-musl/release/x-bot .

FROM scratch
WORKDIR /app
COPY --from=builder /build/x-bot .
