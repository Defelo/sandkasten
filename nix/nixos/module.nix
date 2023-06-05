{
  default,
  lib,
  ...
}: let
  inherit (lib) limits packages time;
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
        redis = mkOption {
          type = types.bool;
          default = true;
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
        use_cgroup = mkOption {
          type = types.bool;
          default = conf.use_cgroup;
        };
        environments = mkOption {
          type = types.anything;
          default = _: [];
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
            Restart = lib.mkDefault "always";
            RestartSec = lib.mkDefault 1;
            OOMPolicy = lib.mkDefault "continue";
          };
          environment = {
            CONFIG_PATH = pkgs.writeText "config.json" (builtins.toJSON ((builtins.removeAttrs cfg [
                "enable"
                "redis"
                "environments"
              ])
              // {
                nsjail_path = "${pkgs.nsjail}/bin/nsjail";
                time_path = "${time}/bin/time";
                environments_path = ["${packages.combined cfg.environments}/share/sandkasten/packages"];
              }
              // (optionalAttrs cfg.redis {
                redis_url = "redis+unix:///${config.services.redis.servers.sandkasten.unixSocket}";
              })));
          };
        };
        services.redis = mkIf cfg.redis {
          servers.sandkasten = {
            enable = true;
            save = [];
          };
        };
      };
    }
