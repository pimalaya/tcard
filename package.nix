# TODO: move this to nixpkgs
# This file aims to be a replacement for the nixpkgs derivation.

{
  buildFeatures ? [ ],
  buildNoDefaultFeatures ? false,
  buildPackages,
  fetchFromGitHub,
  installManPages ? stdenv.buildPlatform.canExecute stdenv.hostPlatform,
  installShellCompletions ? stdenv.buildPlatform.canExecute stdenv.hostPlatform,
  installShellFiles,
  lib,
  rustPlatform,
  stdenv,
}:

let
  emulator = stdenv.hostPlatform.emulator buildPackages;
  exe = stdenv.hostPlatform.extensions.executable;

in
rustPlatform.buildRustPackage {
  inherit buildNoDefaultFeatures;

  pname = "tcard";
  version = "0.0.1";
  cargoHash = "";

  src = fetchFromGitHub {
    owner = "pimalaya";
    repo = "tcard";
    rev = "v0.0.1";
    hash = "";
  };

  nativeBuildInputs = [ installShellFiles ];
  buildFeatures = buildFeatures ++ [ "cli" ];

  postInstall =
    lib.optionalString (lib.hasInfix "wine" emulator) ''
      export WINEPREFIX="''${WINEPREFIX:-$(mktemp -d)}"
      mkdir -p $WINEPREFIX
    ''
    + ''
      mkdir -p $out/share/{completions,man}
      ${emulator} "$out"/bin/tcard${exe} manuals "$out"/share/man
      ${emulator} "$out"/bin/tcard${exe} completions -d "$out"/share/completions bash elvish fish powershell zsh
    ''
    + lib.optionalString installManPages ''
      installManPage "$out"/share/man/*
    ''
    + lib.optionalString installShellCompletions ''
      installShellCompletion --cmd tcard \
        --bash "$out"/share/completions/tcard.bash \
        --fish "$out"/share/completions/tcard.fish \
        --zsh "$out"/share/completions/_tcard
    '';

  meta = {
    description = "CLI & lib to edit vCards as ergonomic TOML, written in Rust";
    mainProgram = "tcard";
    homepage = "https://github.com/pimalaya/tcard";
    changelog = "https://github.com/pimalaya/tcard/blob/master/CHANGELOG.md";
    license = [
      lib.licenses.mit
      lib.licenses.asl20
    ];
    maintainers = with lib.maintainers; [ soywod ];
  };
}
