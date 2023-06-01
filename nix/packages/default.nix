{
  pkgs,
  pkgs-old,
  ...
}: let
  old = ["swift.nix"];
  packages = builtins.listToAttrs (map (name: {
    name = pkgs.lib.removeSuffix ".nix" name;
    value = import (./. + "/${name}") (
      if builtins.elem name old
      then pkgs-old
      else pkgs
    );
  }) (builtins.filter (name: name != "default.nix" && pkgs.lib.hasSuffix ".nix" name) (builtins.attrNames (builtins.readDir ./.))));
in
  packages
