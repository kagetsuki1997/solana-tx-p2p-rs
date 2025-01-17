{ inputs, ... }:
{
  perSystem = { pkgs, system, ... }: {
    apps = with pkgs;
      {
        run-node = {
          type = "app";
          program = toString (writeScript "run-node" ''
            #!${runtimeShell}

            set -o errexit
            set -o errtrace
            set -o nounset
            set -o pipefail
            # Uncomment for debugging purpose
            # set -o xtrace

            export SOLANA_TX_P2P_RELAY_LEADER_DURATION=10s
            export SOLANA_TX_P2P_SIGNING_LEADER_DURATION=15s
            export SOLANA_TX_P2P_SOLANA_PROGRAM_ID=C4zHy4qLZsqTk7jSRTfD2riHdRUJzaWwBfceovJiiBwR
            export SOLANA_TX_P2P_SOLANA_RPC_URL=http://127.0.0.1:8899

            nix run .#solana-tx-p2p -- node
          '');
        };

        run-server = {
          type = "app";
          program = toString (writeScript "run-server" ''
            #!${runtimeShell}

            set -o errexit
            set -o errtrace
            set -o nounset
            set -o pipefail
            # Uncomment for debugging purpose
            # set -o xtrace

            export SOLANA_TX_P2P_API_PORT=8008
            export SOLANA_TX_P2P_GRPC_PORT=50052

            export SOLANA_TX_P2P_RELAY_LEADER_DURATION=10s
            export SOLANA_TX_P2P_SIGNING_LEADER_DURATION=15s
            export SOLANA_TX_P2P_SOLANA_PROGRAM_ID=C4zHy4qLZsqTk7jSRTfD2riHdRUJzaWwBfceovJiiBwR
            export SOLANA_TX_P2P_SOLANA_RPC_URL=http://127.0.0.1:8899

            nix run .#solana-tx-p2p -- server
          '');
        };
      };
  };
}
