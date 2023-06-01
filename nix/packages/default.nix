{
  pkgs,
  pkgs-old,
  ...
}: let
  packages = builtins.listToAttrs (map (name: {
    name = pkgs.lib.removeSuffix ".nix" name;
    value = import (./. + "/${name}") {inherit pkgs pkgs-old;};
  }) (builtins.filter (name: name != "default.nix" && pkgs.lib.hasSuffix ".nix" name) (builtins.attrNames (builtins.readDir ./.))));
in
  packages
