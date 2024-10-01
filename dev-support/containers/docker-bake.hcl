variable "TAG" {
  default = "latest"
}

variable "REPOSITORY" {
  default = "localhost"
}

variable "CARGO_ARGS" {
}

variable "PROFILE" {
  default = "release-lto"
}

group "default" {
  targets = [
    "solana-tx-p2p-server"
  ]
}

target "base" {
  dockerfile = "dev-support/containers/alpine/Dockerfile"
  args = {
    CARGO_ARGS = "${CARGO_ARGS}"
    PROFILE    = "${PROFILE}"
  }
  platforms = ["linux/amd64"]
}

target "solana-tx-p2p-server" {
  inherits = ["base"]
  target   = "solana-tx-p2p-server"
  tags     = ["${REPOSITORY}/solana-tx-p2p/server:${TAG}"]
}

target "verify-docker-context" {
  dockerfile = "dev-support/containers/alpine/verify-docker-context.dockerfile"
  target     = "verify-docker-context"
  tags       = ["${REPOSITORY}/library/verify-docker-context:${TAG}"]
}
