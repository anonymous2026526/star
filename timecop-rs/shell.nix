{ pkgs ? import <nixpkgs> {} }:

let
  llvmPkgs = pkgs.llvmPackages;
  libclangPkg =
    if pkgs.lib.hasAttr "clang-unwrapped" llvmPkgs then llvmPkgs.clang-unwrapped
    else if pkgs.lib.hasAttr "libclang" llvmPkgs then llvmPkgs.libclang
    else llvmPkgs.clang;
  libclangLib = pkgs.lib.getLib libclangPkg;
in
pkgs.mkShell {
  packages = with pkgs; [
    rustc
    cargo
    rustfmt
    clippy
    rust-analyzer
    pkg-config
    openssl

    valgrind
    clang
    libclangPkg
    curl
    unzip
  ];

  shellHook = ''
    export CARGO_HOME="${toString ./.}/.cargo"
    export PATH="$CARGO_HOME/bin:$PATH"

    if [ -z "$LIBCLANG_PATH" ]; then
      _libclang_candidates="
        ${libclangLib}/lib
        ${libclangPkg}/lib
        /nix/store/*-clang-*-lib/lib
        /nix/store/*-libclang-*/lib
      "
      for cand in $_libclang_candidates; do
        if ls "$cand"/libclang.so* >/dev/null 2>&1; then
          export LIBCLANG_PATH="$cand"
          break
        fi
      done
      unset _libclang_candidates
    fi
  '';
}