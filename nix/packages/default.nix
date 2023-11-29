{
  system,
  inputs,
  lib,
}: let
  pkgs = import inputs.nixpkgs {inherit system;};
  pkgs-old = import inputs.nixpkgs-old {inherit system;};
  pkgs-unstable = import inputs.nixpkgs-unstable {inherit system;};

  removeSuffix = pkgs.lib.removeSuffix ".nix";
  isPackage = name: name != "default.nix" && pkgs.lib.hasSuffix ".nix" name;
  packageNames = builtins.filter isPackage (builtins.attrNames (builtins.readDir ./.));
  packages = builtins.listToAttrs (map (name: {
      name = removeSuffix name;
      value = import (./. + "/${name}") {inherit pkgs pkgs-old pkgs-unstable;};
    })
    packageNames);
  derivations = builtins.mapAttrs (id: {
      name,
      version,
      meta ? {},
      default_main_file_name,
      compile_script,
      run_script,
      example ? null,
      test,
      ...
    } @ v: let
      manifest = pkgs.writeText "sandkasten-${id}-${version}-manifest.json" (builtins.toJSON rec {
        sandkasten_version = lib.cargotoml.package.version;
        inherit name version meta default_main_file_name example test;
        compile_script =
          if builtins.isNull v.compile_script
          then null
          else pkgs.writeShellScript "sandkasten-${id}-${version}-compile.sh" v.compile_script;
        run_script = pkgs.writeShellScript "sandkasten-${id}-${version}-run.sh" v.run_script;
        closure =
          (rootPaths: "${pkgs.closureInfo {inherit rootPaths;}}/store-paths") ([run_script]
            ++ (pkgs.lib.optional (compile_script != null) compile_script));
      });
    in
      pkgs.stdenv.mkDerivation {
        inherit version;
        name = id;
        unpackPhase = "true";
        installPhase = ''
          out=$out/share/sandkasten/packages
          mkdir -p $out
          ln -s ${manifest} $out/${id}.json
        '';
      })
  packages;
  merge = n: p:
    pkgs.symlinkJoin {
      name = "sandkasten-${n}-packages";
      paths = p;
    };
in
  derivations
  // rec {
    combined = f: merge "combined" (f (derivations // {inherit all;}));
    all = merge "all" (builtins.attrValues derivations);
  }
