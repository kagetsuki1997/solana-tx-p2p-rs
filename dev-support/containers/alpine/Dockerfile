# syntax=docker/dockerfile:1
# vim: set ft=dockerfile:

# base
FROM rust:alpine3.18 AS base

ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse
ENV RUST_BACKTRACE=1
ENV PROTOC=/usr/bin/protoc
ENV PROTOC_INCLUDE=/usr/include
WORKDIR /build

# layer: packages
RUN <<EOF
#!/usr/bin/env sh
set -o errexit
set -o errtrace
set -o nounset
set -o pipefail
# Uncomment for debugging purpose
# set -o xtrace

apk add --no-cache \
  musl-dev \
  clang-dev \
  lld \
  protobuf-dev \
  file \
  jq \
  make \
  cmake \
  git \
  openssh-client \
  openssl-dev

# FIXME: clang is unable to compile with `--target=x86_64-unknown-linux-musl`
# ln -sf /usr/bin/clang /usr/bin/cc
# ln -sf /usr/bin/clang++ /usr/bin/c++

cc --version
# c++ --version
ld.lld --version
${PROTOC} --version

cargo install cargo-chef --locked

EOF

# layer: toolchain
COPY rust-toolchain.toml .
RUN cargo --version

# planner
FROM base AS planner

COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# builder
FROM base AS builder

ARG CARGO_ARGS
ARG PROFILE=debug

# layer: dependencies
COPY --from=planner /build/recipe.json recipe.json
RUN \
  --mount=type=ssh \
  --mount=type=secret,id=GITHUB_BASIC_TOKEN \
  <<EOF
#!/usr/bin/env sh
set -o errexit
set -o errtrace
set -o nounset
set -o pipefail
# Uncomment for debugging purpose
# set -o xtrace

if [[ -f /run/secrets/GITHUB_BASIC_TOKEN ]]; then
  cat /run/secrets/GITHUB_BASIC_TOKEN
  git config --global \
    url.https://github.com.insteadOf \
    ssh://git@github.com
  git config --global \
    url.https://github.com/.insteadOf \
    git@github.com:
  git config --global \
    http.https://github.com/.extraheader \
    "Authorization: basic $(cat /run/secrets/GITHUB_BASIC_TOKEN)"
else
  mkdir -p -m 0700 ~/.ssh
  ssh-keyscan github.com >> ~/.ssh/known_hosts
fi

if [ "$PROFILE" != "debug" ]; then
  CARGO_ARGS="--profile $PROFILE $CARGO_ARGS"
fi
cargo chef cook $CARGO_ARGS --recipe-path recipe.json

EOF

# layer: binary
COPY . .
RUN <<EOF
#!/usr/bin/env sh
set -o errexit
set -o errtrace
set -o nounset
set -o pipefail
# Uncomment for debugging purpose
set -o xtrace

if [ "$PROFILE" != "debug" ]; then
  CARGO_ARGS="--profile $PROFILE $CARGO_ARGS"
fi
cargo build $CARGO_ARGS

bins=$(cargo metadata --no-deps --format-version 1 |
  jq -r '.packages[].targets[] | select(.kind | map(. == "bin") | any) | .name')

for bin in $bins; do
  cp -v target/"$PROFILE"/"$bin" /usr/local/bin
  file /usr/local/bin/"$bin"
  ldd /usr/local/bin/"$bin"
  /usr/local/bin/"$bin" --version
done

EOF

# solana-tx-p2p-server image
FROM alpine AS solana-tx-p2p-server

COPY --from=builder \
  /usr/local/bin/solana-tx-p2p \
  /usr/bin/

USER 8787:8787

ENTRYPOINT [ "/usr/bin/solana-tx-p2p" ]
