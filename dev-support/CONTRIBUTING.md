# Contributing

## Development

### Build Container Images

You can use all in one `docker buildx bake`:

```bash
docker buildx bake --load -f dev-support/containers/docker-bake.hcl
```

Or using `docker buildx build` with specific docker file and target. Take `verify-docker-context` as an example:

```bash
docker buildx build \
  -f dev-support/containers/alpine/verify-docker-context.dockerfile \
  --target verify-docker-context \
  --tag verify-docker-context .
```

`verify-docker-context` is a container that `ls` your docker dontext:

```bash
docker run --rm verify-docker-context
```

### Formatter and Linter

- `commitlint` with conventional commit preset for Git commit message
- `nixpkgs-fmt` and `deadnix` for `Nix`
- `shfmt` and `shellcheck` for shell script
- `hclfmt` for `HCL`
- `prettier` for `CSS`, `GraphQL`, `HTML`, `JavaScript`, `Markdown`, `YAML`
- `clang-format` for `Protocol Buffers`
- `sleek` for `SQL`

### Git Hooks

It is suggested to install git hook for linting before code committing. This project is configured with [pre-commit](https://pre-commit.com).

Installation steps:

```bash
pre-commit install --install-hooks -t commit-msg -t pre-commit
```

### Tools

- [format](bin/format): Format all files
- [lines-of-code](bin/lines-of-code): Count lines of code in this project
- [lint](bin/lint): Lint all files

### Shell Script

- MUST begin with [Shebang](<https://en.wikipedia.org/wiki/Shebang_(Unix)>) follow by shell options.
- DO NOT write things that JUST run on your devices.
  - [Shebang](<https://en.wikipedia.org/wiki/Shebang_(Unix)>) of a shell script should be `/usr/bin/env {program}` instead of `/bin/{program}`
- Always use `errexit`, `errtrace`, `nounset` and `pipefail`. Fail fast and be aware of exit codes.
  - Use `|| true` on programs that you intentionally let exit non-zero.

Bash script example:

```bash
#!/usr/bin/env bash

set -o errexit
set -o errtrace
set -o nounset
set -o pipefail
# Uncomment for debugging purpose
# set -o xtrace

echo "Hello World"
```

### Commit Message - [Conventional Commits](https://www.conventionalcommits.org/en)

Add scope if possible (ex. `feat(template): Initial commit`).

| Type     | Description                                                                                            |
| -------- | ------------------------------------------------------------------------------------------------------ |
| feat     | A new feature                                                                                          |
| fix      | A bug fix                                                                                              |
| docs     | Documentation only changes                                                                             |
| style    | Changes that do not affect the meaning of the code (white-space, formatting, missing semi-colons, etc) |
| refactor | A code change that neither fixes a bug nor adds a feature                                              |
| perf     | A code change that improves performance                                                                |
| test     | Adding missing tests or correcting existing tests                                                      |
| build    | Changes that affect the build system or external dependencies (example scopes: Cargo, Docker)          |
| ci       | Changes to our CI configuration files and scripts (example scopes: Drone)                              |
| chore    | Other changes that don't modify src or test files                                                      |
