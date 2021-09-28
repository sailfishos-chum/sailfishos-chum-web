FROM rustlang/rust:nightly-alpine as builder
RUN apk add --no-cache \
  libc-dev \
  perl \
  make
WORKDIR /usr/src/sailfishos-chum-web
COPY . .
RUN \
  --mount=type=cache,target=/usr/local/cargo/git \
  --mount=type=cache,target=/usr/local/cargo/registry \
  --mount=type=cache,target=/usr/src/sailfishos-chum-web/target \
  cargo build --release --bin lambda -Z unstable-options --out-dir /tmp

FROM scratch
COPY --from=builder /tmp/lambda /target/functions/
