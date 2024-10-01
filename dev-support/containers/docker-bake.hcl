variable "TAG" {
  default = "latest"
}

variable "REPOSITORY" {
  default = "442277771319.dkr.ecr.us-west-1.amazonaws.com"
}

variable "CARGO_ARGS" {
}

variable "PROFILE" {
  default = "release-lto"
}

group "default" {
  targets = [
    "zta-iam"
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

target "zta-iam" {
  inherits = ["base"]
  target   = "zta-iam"
  tags     = ["${REPOSITORY}/zta/iam:${TAG}"]
}

target "verify-docker-context" {
  dockerfile = "dev-support/containers/alpine/verify-docker-context.dockerfile"
  target     = "verify-docker-context"
  tags       = ["${REPOSITORY}/library/verify-docker-context:${TAG}"]
}
