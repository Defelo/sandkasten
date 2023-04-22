{
  pkgs,
  lib,
  rust,
  ...
}: let
  inherit (lib) cargotoml environments;
in
  pkgs.stdenv.mkDerivation {
    pname = cargotoml.package.name;
    version = cargotoml.package.version;
    unpackPhase = "true";
    installPhase = let
      script = pkgs.writeShellScript "${cargotoml.package.name}.sh" ''
        [[ -n "$ENVIRONMENTS_CONFIG_PATH" ]] || export ENVIRONMENTS_CONFIG_PATH=${environments false}
        ${rust}/bin/${cargotoml.package.name}
      '';
    in "mkdir -p $out/bin && ln -s ${script} $out/bin/${cargotoml.package.name}";
  }
