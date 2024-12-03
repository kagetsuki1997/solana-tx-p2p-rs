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
    - `ls sm` list signed messages
    - `ls tx` list relayed transactions
    - `get tx {signature}` get relayed transaction by signature
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

### Workers / Tasks

- Message Trigger Task
  - produce message every `SOLANA_TX_P2P_MESSAGE_DURATION` and send to Peer Worker
- Heartbeat Trigger Task
  - produce heartbeat every `SOLANA_TX_P2P_HEARTBEAT_DURATION` and send to Peer Worker
- Relayer Election Worker
  - elect relayer in round-robin manner every `SOLANA_TX_P2P_RELAY_LEADER_DURATION`
  - check relayer heartbeat
  - sync relayer info from Peer Worker
  - send new relayer info to Peer Worker
- Signer Election Worker
  - elect signer in round-robin manner every `SOLANA_TX_P2P_SIGNING_LEADER_DURATION`
  - check signer heartbeat
  - sync signer info from Peer Worker
  - send new signer info to Peer Worker
- Peer Worker
  - handle events from other workers
  - handle events from the p2p network and output to other workers
  - handle stdin
  - send messages to the p2p network
  - send signed message to the p2p network
  - send relayed transaction to the p2p network
  - send peer heartbeat to the p2p network
  - send relayer sync info to the p2p network
  - send signer sync info to the p2p network
  - get transaction from Solana network by signature
  - record peers, signed messages, relayed transactions in the memory
- Solana Relayer
  - receive signed message from Peer Worker
  - send signed message to Solana network
  - send relayed transaction to Peer Worker
- Solana Signer
  - receive message from Peer Worker
  - sign message
  - send signed message to Peer Worker

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

- run local Solana local emulator

```
solana config set -ul
solana-test-validator
```

- deploy program

```
solana program deploy ./target/deploy/solana_hello_world.so
```

- update the program id to `SOLANA_TX_P2P_SOLANA_PROGRAM_ID` in solana-tx-p2p-node.sh and solana-tx-p2p-server.sh

- check log

```
solana logs
```

### Run

Open multiple terminals and run nodes or server

- Run node

```
./solana-tx-p2p-node.sh
```

- Run node with http server

```
./solana-tx-p2p-server.sh
```

## Contributing

See [CONTRIBUTING.md](dev-support/CONTRIBUTING.md).

## License

See [LICENSE](LICENSE).
