# syntax=docker/dockerfile:1
# vim: set ft=dockerfile:

FROM docker.io/library/alpine:latest AS verify-docker-context

RUN <<EOF
#!/usr/bin/env sh

set -o errexit
set -o errtrace
set -o nounset
set -o pipefail
# Uncomment for debugging purpose
# set -o xtrace

apk add --no-cache \
  exa \
  tzdata

EOF

USER 8787:8787
ENV TZ=UTC

WORKDIR /project
COPY . /project/

ENTRYPOINT [ "exa", "--all", "--long", "--tree" ]
