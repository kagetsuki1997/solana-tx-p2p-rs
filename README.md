# Solana Transactions with P2P Network

## Features

- Node
  - P2P Network with `libp2p`
  - Elect Solana Transaction Signer in round-robin manner
  - Elect Solana Transaction Relayer in round-robin manner
  - Handle peer joining and leaving
  - Send transaction to Solana
  - Stdin
  - `ls p` list peers
  - `ls s` list signed messages
  - `ls t` list relayed messages
- Server
  - [gRPC](proto/p2p)
  - [RESTful API](docs/openapi)
  - Run Node at the same time

## Project

### Project Structure

- [`dev-support`](dev-support) contains development utilities
  - [`dev-support/bin`](dev-support/bin) contains tools which will be used through development process
  - [`dev-support/ci-bin`](dev-support/ci-bin) contains scripts used by CI
  - [`dev-support/containers`](dev-support/containers) contains the container related definitions
  - [`dev-support/flake-modules`](dev-support/flake-modules) contains Nix flake modules (ex. development environment)
- [`openapi`](docs/openapi) contains OpenAPI docs
- [`proto`](proto) contains protobufs
- [`solana-hello-world`](solana-hello-world) a simple solana program for transaction
- [`solana-tx-p2p`](solana-tx-p2p) the main program

### Prepare Solana

[Reference](https://solana.com/docs/programs/rust)

- make sure solana-cli install

```
agave-install init 2.1.1
```

- build program

```
cargo build-sbf
```

- run local Solana cluster

```
solana config set -ul
solana-test-validator
```

- deploy program

```
solana program deploy ./target/deploy/solana_hello_world.so
```

- check log

```
solana logs
```

### Run

Run node

```
./solana-tx-p2p-node.sh
```

Run node with http server

```
./solana-tx-p2p-server.sh
```

## Contributing

See [CONTRIBUTING.md](dev-support/CONTRIBUTING.md).

## License

See [LICENSE](LICENSE).
