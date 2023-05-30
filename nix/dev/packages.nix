{
  pkgs,
  lib,
  ...
}: let
  inherit (lib) envs config;
in
  builtins.mapAttrs (k: {
    name,
    version,
    compile_script,
    run_script,
    ...
  }:
    pkgs.stdenv.mkDerivation {
      inherit version;
      name = k;
      unpackPhase = "true";
      installPhase = let
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
  (envs false)
