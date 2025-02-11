{
  description = "netns-exec - run a process in a Linux network namespace";

  inputs = {
    naersk.url = "github:nix-community/naersk/master";
    utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs = { self, nixpkgs, utils, naersk}:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {inherit system; };
        naersk-lib = pkgs.callPackage naersk { };
      in {
        defaultPackage = naersk-lib.buildPackage ./.;
        devShell = with pkgs; mkShell {
          buildInputs = [ rustc cargo rustPackages.clippy rustfmt bashInteractive ];
          RUST_SRC_PATH = rustPlatform.rustLibSrc;
        };
      }
    ) 
  ;
}
