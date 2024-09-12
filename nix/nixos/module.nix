{
  lib,
  self,
  ...
}: let
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
        settings = mkOption {
          type = types.attrs;
          default = {};
        };
      };
      config = mkIf cfg.enable {
        systemd.services.sandkasten = {
          wantedBy = ["multi-user.target"];
          serviceConfig = {
            ExecStart = "${self.packages.${pkgs.system}.sandkasten}/bin/sandkasten";
            StateDirectory = "sandkasten";
            PrivateTmp = true;
            Restart = lib.mkDefault "always";
            RestartSec = lib.mkDefault 1;
            OOMPolicy = lib.mkDefault "continue";
          };
          environment = let
            opts = {
              nsjail_path = "${pkgs.nsjail}/bin/nsjail";
              time_path = "${self.packages.${pkgs.system}.time}/bin/time";
              environments_path = ["${self.packages.${pkgs.system}.packages.combined cfg.environments}/share/sandkasten/packages"];
              programs_dir = "/var/lib/sandkasten/programs";
              jobs_dir = "/tmp/sandkasten/jobs";
              base_resource_usage_permits = (conf // cfg.settings).max_concurrent_jobs;
            };
          in {
            CONFIG_PATH = pkgs.writeText "sandkasten-config.json" (builtins.toJSON (builtins.foldl' lib.recursiveUpdate {} [conf opts cfg.settings]));
          };
        };
      };
    }
