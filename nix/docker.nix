{
  pkgs,
  lib,
  rust,
  ...
}: let
  inherit (lib) cargotoml environments;
in
  pkgs.dockerTools.buildLayeredImage {
    name = cargotoml.package.name;
    tag = cargotoml.package.version;
    contents = with pkgs; [
      nsjail
      coreutils-full
      bashInteractive
      rust
    ];
    config = {
      User = "65534:65534";
      Entrypoint = ["${rust}/bin/${cargotoml.package.name}"];
      Env = [
        "ENVIRONMENTS_CONFIG_PATH=${environments false}"
        "USE_CGROUP=false"
      ];
    };
  }
