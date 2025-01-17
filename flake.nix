{
  description = "Development environment";

  inputs = {
    flake-parts = {
      url = github:hercules-ci/flake-parts;
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };
    nixpkgs.url = github:NixOS/nixpkgs/nixos-unstable;
  };

  outputs = inputs @ { flake-parts, ... }:
    flake-parts.lib.mkFlake
      { inherit inputs; }
      {
        imports = [
          ./dev-support/flake-modules/shell.nix
          ./dev-support/flake-modules/default.nix
          ./dev-support/flake-modules/script.nix
        ];

        systems = [ "aarch64-darwin" "aarch64-linux" "x86_64-darwin" "x86_64-linux" ];
      };

}
