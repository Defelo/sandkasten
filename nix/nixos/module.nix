{
  default,
  lib,
  ...
}: let
  inherit (lib) packages time;
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
      options.services.sandkasten = {
        enable = mkEnableOption "sandkasten";
        environments = mkOption {
          type = types.anything;
          default = _: [];
        };
        redis = mkOption {
          type = types.bool;
          default = true;
        };
        settings = mkOption {
          type = types.attrs;
          default = {};
        };
      };
      config = mkIf cfg.enable {
        systemd.services.sandkasten = {
          wantedBy = ["multi-user.target"];
          serviceConfig = {
            ExecStart = "${default}/bin/sandkasten";
            StateDirectory = "sandkasten";
            PrivateTmp = true;
            Restart = lib.mkDefault "always";
            RestartSec = lib.mkDefault 1;
            OOMPolicy = lib.mkDefault "continue";
          };
          environment = let
            opts =
              {
                nsjail_path = "${pkgs.nsjail}/bin/nsjail";
                time_path = "${time}/bin/time";
                environments_path = ["${packages.combined cfg.environments}/share/sandkasten/packages"];
                programs_dir = "/var/lib/sandkasten/programs";
                jobs_dir = "/tmp/sandkasten/jobs";
                base_resource_usage_permits = (conf // cfg.settings).max_concurrent_jobs;
              }
              // lib.optionalAttrs cfg.redis {
                redis_url = "redis+unix:///${config.services.redis.servers.sandkasten.unixSocket}";
              };
          in {
            CONFIG_PATH = pkgs.writeText "sandkasten-config.json" (builtins.toJSON (builtins.foldl' lib.recursiveUpdate {} [conf opts cfg.settings]));
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
