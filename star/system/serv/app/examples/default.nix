{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  packages = with pkgs; [
    rustup
    clang
    cmake
    sqlite
    openssl
  ];

  shellHook = ''
    set -euo pipefail

    mkdir -p $(pwd)/certs

    openssl req -x509 -newkey ed25519 -nodes \
      -keyout certs/server.key \
      -out certs/server.crt \
      -days 365 \
      -subj "/CN=localhost"

    echo "Generated certs/server.crt and certs/server.key"

    export TLS_CERT_PATH=$(pwd)/certs/server.crt
    export TLS_KEY_PATH=$(pwd)/certs/server.key
    #export TLS_CERT_PATH=$(pwd)/certs/server.crt
    #export TLS_KEY_PATH=$(pwd)/certs/server.crt
  '';
}