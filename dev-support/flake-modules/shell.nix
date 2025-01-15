{ inputs, ... }:
{
  perSystem = { pkgs, system, ... }: {
    _module.args.pkgs = import inputs.nixpkgs {
      inherit system;
      config.allowUnfree = true;
    };

    devShells.default =
      with pkgs;
      let
        llvmPackages = llvmPackages_18;
        nodejs = nodejs_22;
        # solana = callPackage ./solana-cli/package.nix { };
        solanaVersion = "2.1.1";
        solana = (solana-cli.override {
          rocksdb_8_3 = rocksdb_8_11;
          solanaPkgs = [
            "cargo-build-sbf"
            "cargo-test-sbf"
            "solana"
            "solana-bench-tps"
            "solana-faucet"
            "solana-gossip"
            "agave-install"
            "solana-keygen"
            "agave-ledger-tool"
            "solana-log-analyzer"
            "solana-net-shaper"
            "agave-validator"
            "solana-test-validator"
          ] ++ [
            # XXX: Ensure `solana-genesis` is built LAST!
            # See https://github.com/solana-labs/solana/issues/5826
            "solana-genesis"
          ];
        }).overrideAttrs (drv: rec {
          version = "${solanaVersion}";
          hash = "sha256-cWdtEzs1ROHpNKscDszuIUA0C8IMoEk2G/VzcntlC3A=";
          src = fetchFromGitHub {
            owner = "anza-xyz";
            repo = "agave";
            rev = "v${version}";
            inherit hash;
          };
          cargoDeps = rustPlatform.importCargoLock {
            lockFile = ./solana-cli/Cargo.lock;
            outputHashes = {
              "crossbeam-epoch-0.9.5" = "sha256-Jf0RarsgJiXiZ+ddy0vp4jQ59J9m0k3sgXhWhCdhgws=";
              "tokio-1.29.1" = "sha256-Z/kewMCqkPVTXdoBcSaFKG5GSQAdkdpj3mAzLLCjjGk=";
            };
          };
          postInstall = lib.optionalString (stdenv.buildPlatform.canExecute stdenv.hostPlatform) ''
            installShellCompletion --cmd solana \
              --bash <($out/bin/solana completion --shell bash) \
              --fish <($out/bin/solana completion --shell fish)

            mkdir -p $out/bin/sdk/sbf
            cp -a ./sdk/sbf/* $out/bin/sdk/sbf/
          '';
        });
      in
      mkShellNoCC {
        packages =
          [
            direnv
            nix-direnv

            gitAndTools.git-extras
            gitAndTools.pre-commit
            nodejs.pkgs."@commitlint/cli"
            tokei

            treefmt
            nixpkgs-fmt
            deadnix
            shfmt
            shellcheck
            nodejs.pkgs.prettier
            nodejs.pkgs.sql-formatter
            taplo
            hclfmt
            llvmPackages.clang-tools
            codespell

            docker
            docker-buildx

            terraform

            protobuf

            llvmPackages.clang
            llvmPackages.lld
            pkg-config

            rustup
            sccache
            cargo-audit
            cargo-deny
            cargo-nextest

            llvmPackages.libclang.lib
            openssl.dev

            sqlx-cli

            solana
          ] ++ lib.optionals stdenv.isDarwin [
            libiconv
            darwin.apple_sdk.frameworks.Security
            darwin.apple_sdk.frameworks.SystemConfiguration
          ] ++ lib.optionals (!stdenv.isDarwin) [
            cargo-llvm-cov
          ];

        shellHook = ''
          export PATH=$PWD/dev-support/bin:$PATH
          # export PATH=~/.local/share/solana/install/active_release/bin:$PATH

          # Workaround for solana sdk platform tools installation
          export SBF_SDK_PATH=$HOME/.local/share/solana/install/releases/${solanaVersion}/solana-release/bin/sdk/sbf
          if [ ! -d "$SBF_SDK_PATH" ]; then
            mkdir -p $SBF_SDK_PATH
            cp -a ${solana.outPath}/bin/sdk/sbf/* $SBF_SDK_PATH/
          fi
        '';

        COMMITLINT_PRESET = "${nodejs.pkgs."@commitlint/config-conventional"}/lib/node_modules/@commitlint/config-conventional/lib/index.js";

        LIBCLANG_PATH = "${llvmPackages.libclang.lib}/lib";

        PROTOC = "${protobuf}/bin/protoc";
        PROTOC_INCLUDE = "${protobuf}/include";

        RUST_BACKTRACE = 1;
      };
  };
}
