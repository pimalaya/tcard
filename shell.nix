{
  nixpkgs ? <nixpkgs>,
  system ? builtins.currentSystem,
  pkgs ? import nixpkgs { inherit system; },
  pimalaya ? import (fetchTarball "https://github.com/pimalaya/nix/archive/master.tar.gz"),
  ...
}@args:

let
  inherit (pkgs) cargo-deny;
  shell = pimalaya.mkShell (removeAttrs args [ "pimalaya" ]);

in
shell.overrideAttrs (prev: {
  buildInputs = (prev.buildInputs or [ ]) ++ [
    cargo-deny
  ];
})
