{ inputs, ... }:
{
  perSystem = { pkgs, system, ... }: {
    packages = with pkgs;
      {
        solana-tx-p2p = let configFile = lib.importTOML ../../.cargo/config.toml; in rustPlatform.buildRustPackage
          rec {
            pname = "solana-tx-p2p";
            version = "0.0.1";

            src = lib.cleanSource ../../.;
            cargoBuildFlags = [
              "--bin"
              "solana-tx-p2p"
            ];
            RUSTFLAGS = configFile.build.rustflags;

            cargoLock = {
              lockFile = ../../Cargo.lock;
            };

            nativeBuildInputs = [
              protobuf
              pkg-config
              llvmPackages.clang
              llvmPackages.lld
              llvmPackages.libclang.lib
            ];
            buildInputs = [
              openssl.dev
            ] ++ lib.optionals stdenv.isDarwin [
              libiconv
              darwin.apple_sdk.frameworks.Security
              darwin.apple_sdk.frameworks.SystemConfiguration
            ] ++ lib.optionals (!stdenv.isDarwin) [
              cargo-llvm-cov
            ];


            OPENSSL_NO_VENDOR = 1;

            meta = with lib;{
              mainProgram = "solana-tx-p2p";
            };
          };
      };
  };
}
