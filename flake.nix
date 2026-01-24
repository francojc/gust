{
  description = "gust - A terminal-based weather dashboard in Rust";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    rust-overlay,
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      overlays = [(import rust-overlay)];
      pkgs = import nixpkgs {inherit system overlays;};

      rustToolchain = pkgs.rust-bin.stable.latest.default.override {
        extensions = ["rust-src" "rust-analyzer" "clippy" "rustfmt"];
      };

      # Development packages
      packages = with pkgs; [
        rustToolchain
        cargo-watch
        cargo-edit
        cargo-insta
        direnv
        gh
        git
      ];
    in {
      devShells.default = pkgs.mkShell {
        buildInputs = packages;
        shellHook = ''
          echo "gust development environment activated"
          echo "Rust version: $(rustc --version)"
        '';

        # Required for openssl/reqwest on some systems
        OPENSSL_DIR = "${pkgs.openssl.dev}";
        OPENSSL_LIB_DIR = "${pkgs.openssl.out}/lib";
        PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
      };
    });
}
