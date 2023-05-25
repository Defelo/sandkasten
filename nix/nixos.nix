{
  default,
  lib,
  ...
}: let
  inherit (lib) limits;
  conf = lib.config;
in
  {
    pkgs,
    lib,
    config,
    ...
  }:
    with lib; let
      cfg = config.services.sandkasten;
    in {
      imports = [];
      options.services.sandkasten = let
        limit_types =
          (builtins.mapAttrs (k: v: types.int) limits.u64)
          // (builtins.mapAttrs (k: v: types.bool) limits.bool);
      in {
        enable = mkEnableOption "sandkasten";
        uid = mkOption {
          type = types.nullOr types.int;
          default = null;
        };
        gid = mkOption {
          type = types.nullOr types.int;
          default = null;
        };
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
        redis_url = mkOption {
          type = types.str;
          default = conf.redis_url;
        };
        cache_ttl = mkOption {
          type = types.int;
          default = conf.cache_ttl;
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
        base_resource_usage_runs = mkOption {
          type = types.int;
          default = conf.base_resource_usage_runs;
        };
        compile_limits = builtins.mapAttrs (k: v:
          mkOption {
            type = limit_types.${k};
            default = v;
          })
        conf.compile_limits;
        run_limits = builtins.mapAttrs (k: v:
          mkOption {
            type = limit_types.${k};
            default = v;
          })
        conf.run_limits;
      };
      config = mkIf cfg.enable {
        systemd.services.sandkasten = {
          wantedBy = ["multi-user.target"];
          serviceConfig = {
            ExecStart = "${default}/bin/sandkasten";
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
          inherit (cfg) uid;
          group = "sandkasten";
          isSystemUser = true;
        };
        users.groups.sandkasten = {
          inherit (cfg) gid;
        };
      };
    }
