{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/release-22.11";
  };

  outputs = {nixpkgs, ...}: let
    system = "x86_64-linux";
    pkgs = import nixpkgs {inherit system;};
    time = import ./nix/time pkgs;
    packages = import ./nix/packages {inherit pkgs;};
    envs = dev:
      builtins.mapAttrs (k: v:
        {
          inherit (v) name version;
          compile_script =
            if builtins.isNull v.compile_script
            then null
            else pkgs.writeShellScript "${k}-compile.sh" v.compile_script;
          run_script = pkgs.writeShellScript "${k}-run.sh" v.run_script;
        }
        // pkgs.lib.optionalAttrs dev {inherit (v) test;})
      packages;
    environments = dev:
      pkgs.writeText "environments.json" (builtins.toJSON {
        nsjail_path = "${pkgs.nsjail}/bin/nsjail";
        time_path = "${time}/bin/time";
        environments = envs dev;
      });
  in {
    packages.${system} = rec {
      packages = builtins.mapAttrs (k: {
        name,
        version,
        compile_script,
        run_script,
      }:
        pkgs.stdenv.mkDerivation {
          inherit name version;
          unpackPhase = "true";
          installPhase = let
            sandbox = file:
              pkgs.writeShellScript "run-in-sandbox.sh"
              ''${pkgs.nsjail}/bin/nsjail -q --cwd /box -B $PWD/box:/box -B $PWD/program:/out -E MAIN -R /nix/store -R $PWD/program:/program -T /tmp --rlimit_as hard -- ${file} "$@"'';
          in ''
            mkdir -p $out/bin
            ${
              if compile_script != null
              then "ln -s ${sandbox compile_script} $out/bin/${k}-compile.sh"
              else ""
            }
            ln -s ${sandbox run_script} $out/bin/${k}-run.sh
          '';
        })
      (envs false);
      rust = pkgs.rustPlatform.buildRustPackage {
        name = "sandkasten";
        src = pkgs.stdenv.mkDerivation {
          name = "sandkasten-src";
          src = ./src;
          installPhase = let
            files = {
              "Cargo.toml" = ./Cargo.toml;
              "Cargo.lock" = ./Cargo.lock;
              src = ./src;
            };
          in
            builtins.foldl' (acc: k: acc + " && ln -s ${files.${k}} $out/${k}") "mkdir -p $out" (builtins.attrNames files);
        };
        cargoLock.lockFile = ./Cargo.lock;
      };
      docker = pkgs.dockerTools.buildLayeredImage {
        name = "ghcr.io/defelo/sandkasten";
        tag = "latest";
        contents = with pkgs; [
          nsjail
          coreutils-full
          bash
        ];
        config = {
          User = "65534:65534";
          Entrypoint = ["${rust}/bin/sandkasten"];
          Env = ["ENVIRONMENTS_CONFIG_PATH=${environments false}"];
        };
      };
      default = docker;
    };
    devShells.${system} = let
      test-env = {
        ENVIRONMENTS_CONFIG_PATH = environments true;
        PACKAGES_TEST_SRC = pkgs.writeText "packages_test_src.rs" (builtins.foldl' (acc: pkg:
          acc
          + ''
            #[tokio::test]
            #[ignore]
            async fn test_${pkg}() {
              test_package("${pkg}").await;
            }
          '') "" (builtins.attrNames packages));
        CONFIG_PATH = pkgs.writeText "config.json" (builtins.toJSON (let
          config = builtins.fromTOML (builtins.readFile ./config.toml);
        in
          config
          // {
            host = "127.0.0.1";
            port = 8000;
            server = "/";
            programs_dir = "programs";
            jobs_dir = "jobs";
          }));
      };
      test-script = pkgs.writeShellScript "integration-tests.sh" ''
        rm -rf programs jobs
        cargo build --locked
        cargo run --locked &
        pid=$!
        sleep 1
        cargo test --locked --all-features --all-targets --no-fail-fast -- --ignored
        out=$?
        kill -9 $pid
        exit $out
      '';
      scripts = pkgs.stdenv.mkDerivation {
        name = "scripts";
        unpackPhase = "true";
        installPhase = "mkdir -p $out/bin && ln -s ${test-script} $out/bin/integration-tests";
      };
    in {
      default = pkgs.mkShell ({
          packages = [pkgs.nsjail time scripts];
          RUST_LOG = "info,sandkasten=trace,difft=off";
        }
        // test-env);
      test = pkgs.mkShell ({packages = [scripts];} // test-env);
    };
  };
}
