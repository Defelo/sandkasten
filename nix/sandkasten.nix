{
  runCommandNoCCLocal,
  rustPlatform,
}: let
  inherit (fromTOML (builtins.readFile ../Cargo.toml)) package;
  files = {
    "Cargo.toml" = ../Cargo.toml;
    "Cargo.lock" = ../Cargo.lock;
    src = ../src;
    client = ../client;
  };
in
  rustPlatform.buildRustPackage {
    pname = package.name;
    version = package.version;
    src = runCommandNoCCLocal "src" {} (builtins.foldl' (acc: k: acc + " && cp -r ${files.${k}} $out/${k}") "mkdir -p $out" (builtins.attrNames files));
    cargoLock.lockFile = ../Cargo.lock;
    doCheck = false;
  }
