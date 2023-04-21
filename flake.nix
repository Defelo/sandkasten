{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/release-22.11";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    fenix,
    naersk,
    ...
  }: let
    system = "x86_64-linux";
    pkgs = import nixpkgs {inherit system;};
    time = import ./nix/time pkgs;
    cargotoml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
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
              pkgs.writeShellScript "sandbox.sh"
              ''
                ${pkgs.nsjail}/bin/nsjail -q \
                  --user 65534 \
                  --group 65534 \
                  --hostname box \
                  --cwd /box \
                  -R /nix/store \
                  -R $PWD/box:/box \
                ${
                  if build
                  then "-B $PWD/program:/program"
                  else "-R $PWD/program:/program"
                } \
                  -m none:/tmp:tmpfs:size=${toString limits.tmpfs}M \
                  -R /dev/null \
                  -R /dev/urandom \
                  -s /proc/self/fd:/dev/fd \
                  -s /dev/null:/etc/passwd \
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
      rust = let
        toolchain = with fenix.packages.${system};
          combine [
            stable.rustc
            stable.cargo
            targets.x86_64-unknown-linux-musl.stable.rust-std
          ];
      in
        (naersk.lib.${system}.override {
          cargo = toolchain;
          rustc = toolchain;
        })
        .buildPackage {
          src = pkgs.stdenv.mkDerivation {
            name = "src";
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
          CARGO_BUILD_TARGET = "x86_64-unknown-linux-musl";
        };
      docker = pkgs.dockerTools.buildLayeredImage {
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
          Env = ["ENVIRONMENTS_CONFIG_PATH=${environments false}"];
        };
      };
      default = pkgs.stdenv.mkDerivation {
        pname = cargotoml.package.name;
        version = cargotoml.package.version;
        unpackPhase = "true";
        installPhase = let
          script = pkgs.writeShellScript "${cargotoml.package.name}.sh" ''
            [[ -n "$ENVIRONMENTS_CONFIG_PATH" ]] || export ENVIRONMENTS_CONFIG_PATH=${environments false}
            ${rust}/bin/${cargotoml.package.name}
          '';
        in "mkdir -p $out/bin && ln -s ${script} $out/bin/${cargotoml.package.name}";
      };
    };
    nixosModules.sandkasten = {
      pkgs,
      lib,
      config,
      ...
    }:
      with lib; let
        inherit (pkgs) system;
        cfg = config.services.sandkasten;
      in {
        imports = [];
        options.services.sandkasten = let
          conf = builtins.fromTOML (builtins.readFile ./config.toml);
        in {
          enable = mkEnableOption "sandkasten";
          host = mkOption {
            type = types.str;
            default = "0.0.0.0";
          };
          port = mkOption {
            type = types.port;
            default = 8000;
          };
          server = mkOption {
            type = types.str;
            default = "/";
          };
          programs_dir = mkOption {
            type = types.path;
            default = "/srv/sandkasten/programs";
          };
          jobs_dir = mkOption {
            type = types.path;
            default = "/tmp/.sandkasten/jobs";
          };
          program_ttl = mkOption {
            type = types.int;
            default = conf.program_ttl;
          };
          prune_programs_interval = mkOption {
            type = types.int;
            default = conf.prune_programs_interval;
          };
          max_concurrent_jobs = mkOption {
            type = types.int;
            default = conf.max_concurrent_jobs;
          };
          compile_limits = builtins.mapAttrs (k: v:
            mkOption {
              type = types.int;
              default = v;
            })
          conf.compile_limits;
          run_limits = builtins.mapAttrs (k: v:
            mkOption {
              type = types.int;
              default = v;
            })
          conf.run_limits;
        };
        config = mkIf cfg.enable {
          systemd.services.sandkasten = {
            wantedBy = ["multi-user.target"];
            serviceConfig = {
              ExecStart = "${self.packages.${system}.default}/bin/sandkasten";
              ExecStartPre = [
                "+${pkgs.coreutils}/bin/mkdir -p ${cfg.programs_dir}"
                "+${pkgs.coreutils}/bin/mkdir -p ${cfg.jobs_dir}"
                "+${pkgs.coreutils}/bin/chown sandkasten:sandkasten ${cfg.programs_dir}"
                "+${pkgs.coreutils}/bin/chown sandkasten:sandkasten ${cfg.jobs_dir}"
              ];
              User = "sandkasten";
              Group = "sandkasten";
              Restart = "always";
              RestartSec = 0;
              OOMPolicy = "continue";
            };
            environment = {
              CONFIG_PATH = pkgs.writeText "config.json" (builtins.toJSON cfg);
            };
          };
          users.users.sandkasten = {
            group = "sandkasten";
            isSystemUser = true;
          };
          users.groups.sandkasten = {};
        };
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
        ENVIRONMENTS_LIST_SRC = pkgs.writeText "environments_list_src.rs" ''
          const ENVIRONMENTS: &[&str] = &[${builtins.foldl' (acc: x: acc + ''"${x}", '') "" (builtins.attrNames packages)}];
        '';
        LIMITS_TEST_SRC = let
          minvals = {
            cpus = 1;
            time = 1;
            memory = 1;
            tmpfs = 0;
            filesize = 1;
            file_descriptors = 1;
            processes = 1;
            stdout_max_size = 0;
            stderr_max_size = 0;
          };
        in
          pkgs.writeText "limits_test_src.rs" (let
            numeric = builtins.attrNames minvals;
            compile.network =
              if config.compile_limits.network
              then "any::<bool>()"
              else "Just(false)";
            run.network =
              if config.run_limits.network
              then "any::<bool>()"
              else "Just(false)";
          in ''
            prop_compose! {
                fn compile_limits() (
                    ${builtins.foldl' (acc: x: acc + "${x} in option::of(${toString minvals.${x}}u64..=${toString config.compile_limits.${x}}), ") "network in option::of(${compile.network}), " numeric}
                ) -> LimitsOpt {
                    LimitsOpt {
                      network,
                      ${builtins.foldl' (acc: x: acc + "${x}, ") "" numeric}
                    }
                }
            }
            prop_compose! {
                fn run_limits() (
                    ${builtins.foldl' (acc: x: acc + "${x} in option::of(${toString minvals.${x}}u64..=${toString config.run_limits.${x}}), ") "network in option::of(${run.network}), " numeric}
                ) -> LimitsOpt {
                    LimitsOpt {
                      network,
                      ${builtins.foldl' (acc: x: acc + "${x}, ") "" numeric}
                    }
                }
            }
          '');
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
