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
          inherit version;
          name = k;
          unpackPhase = "true";
          installPhase = let
            config = builtins.fromTOML (builtins.readFile ./config.toml);
            sandbox = build: file: let
              limits =
                if build
                then config.compile_limits
                else config.run_limits;
            in
              pkgs.writeShellScript "run-in-sandbox.sh"
              ''
                ${pkgs.nsjail}/bin/nsjail -q \
                  --cwd /box \
                  -R $PWD/box:/box \
                  -${
                  if build
                  then "B"
                  else "R"
                } $PWD/program:/program \
                  -E MAIN \
                  -R /nix/store \
                  -m none:/tmp:tmpfs:size=${toString limits.tmpfs}M \
                  -R /dev/null \
                  -R /dev/urandom \
                  -s /dev/null:/etc/passwd \
                  -s /proc/self/fd:/dev/fd \
                  --max_cpus ${toString limits.cpus} \
                  --time_limit ${toString limits.time} \
                  --rlimit_as ${toString limits.memory} \
                  --rlimit_fsize ${toString limits.filesize} \
                  --rlimit_nofile ${toString limits.file_descriptors} \
                  --rlimit_nproc ${toString limits.processes} \
                  -- ${file} "$@"
              '';
          in ''
            mkdir -p $out/bin
            ${
              if compile_script != null
              then "ln -s ${sandbox true compile_script} $out/bin/${k}-compile.sh"
              else ""
            }
            ln -s ${sandbox false run_script} $out/bin/${k}-run.sh
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
      test-env = let
        config = builtins.fromTOML (builtins.readFile ./config.toml);
      in {
        ENVIRONMENTS_CONFIG_PATH = environments true;
        PACKAGES_TEST_SRC = pkgs.writeText "packages_test_src.rs" (builtins.foldl' (acc: pkg:
          acc
          + ''
            #[test]
            #[ignore]
            fn test_${pkg}() {
              test_package("${pkg}");
            }
          '') "" (builtins.attrNames packages));
        LIMITS_TEST_SRC = pkgs.writeText "limits_test_src.rs" ''
          prop_compose! {
              fn compile_limits() (
                  ${builtins.foldl' (acc: x: acc + "${x} in option::of(0u64..=${toString config.compile_limits.${x}}), ") "" (builtins.attrNames config.compile_limits)}
              ) -> LimitsOpt {
                  LimitsOpt {
                    ${builtins.foldl' (acc: x: acc + "${x}, ") "" (builtins.attrNames config.compile_limits)}
                  }
              }
          }
          prop_compose! {
              fn run_limits() (
                  ${builtins.foldl' (acc: x: acc + "${x} in option::of(0u64..=${toString config.run_limits.${x}}), ") "" (builtins.attrNames config.run_limits)}
              ) -> LimitsOpt {
                  LimitsOpt {
                    ${builtins.foldl' (acc: x: acc + "${x}, ") "" (builtins.attrNames config.run_limits)}
                  }
              }
          }
        '';
        CONFIG_PATH = pkgs.writeText "config.json" (builtins.toJSON (config
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
        cargo build -r --locked
        cargo run -r --locked &
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
