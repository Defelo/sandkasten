{
  pkgs,
  lib,
  rust,
  ...
}: let
  inherit (lib) cargotoml;
in
  pkgs.stdenv.mkDerivation {
    pname = cargotoml.package.name;
    version = cargotoml.package.version;
    unpackPhase = "true";
    installPhase = let
      script = pkgs.writeShellScript "${cargotoml.package.name}.sh" ''
        ${rust}/bin/${cargotoml.package.name}
      '';
    in "mkdir -p $out/bin && ln -s ${script} $out/bin/${cargotoml.package.name}";
  }
