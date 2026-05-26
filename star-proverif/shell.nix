let pkgs = import <nixpkgs> {};

in pkgs.mkShell rec {
  name = "star-proverif";

  buildInputs = with pkgs; [
    proverif gnumake
  ];
}