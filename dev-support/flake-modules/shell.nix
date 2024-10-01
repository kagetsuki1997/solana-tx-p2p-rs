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
          ] ++ lib.optionals stdenv.isDarwin [
            libiconv
            darwin.apple_sdk.frameworks.Security
            darwin.apple_sdk.frameworks.SystemConfiguration
          ] ++ lib.optionals (!stdenv.isDarwin) [
            cargo-llvm-cov
          ];

        shellHook = ''
          export PATH=$PWD/dev-support/bin:~/.local/share/solana/install/active_release/bin:$PATH
        '';

        COMMITLINT_PRESET = "${nodejs.pkgs."@commitlint/config-conventional"}/lib/node_modules/@commitlint/config-conventional/lib/index.js";

        LIBCLANG_PATH = "${llvmPackages.libclang.lib}/lib";

        PROTOC = "${protobuf}/bin/protoc";
        PROTOC_INCLUDE = "${protobuf}/include";

        RUST_BACKTRACE = 1;
      };
  };
}
