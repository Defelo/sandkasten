{
  stdenv,
  system,
  inputs,
}: let
  toolchain = with inputs.fenix.packages.${system};
    combine [
      stable.rustc
      stable.cargo
      targets.x86_64-unknown-linux-musl.stable.rust-std
    ];
in
  (inputs.naersk.lib.${system}.override {
    cargo = toolchain;
    rustc = toolchain;
  })
  .buildPackage {
    src = stdenv.mkDerivation {
      name = "src";
      unpackPhase = "true";
      installPhase = let
        files = {
          "Cargo.toml" = ../Cargo.toml;
          "Cargo.lock" = ../Cargo.lock;
          src = ../src;
          client = ../client;
        };
      in
        builtins.foldl' (acc: k: acc + " && cp -r ${files.${k}} $out/${k}") "mkdir -p $out" (builtins.attrNames files);
    };
    CARGO_BUILD_TARGET = "x86_64-unknown-linux-musl";
  }
