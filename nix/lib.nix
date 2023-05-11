{pkgs, ...}: rec {
  time = import ./time pkgs;
  cargotoml = builtins.fromTOML (builtins.readFile ../Cargo.toml);
  config = builtins.fromTOML (builtins.readFile ../config.toml);
  packages = import ./packages {inherit pkgs;};
  limits = {
    u64 = {
      cpus = {min = 1;};
      time = {min = 1;};
      memory = {min = 1;};
      tmpfs = {min = 0;};
      filesize = {min = 1;};
      file_descriptors = {min = 1;};
      processes = {min = 1;};
      stdout_max_size = {min = 0;};
      stderr_max_size = {min = 0;};
    };
    bool = {
      network = {};
    };
  };
  envs = dev:
    builtins.mapAttrs (k: v:
      rec {
        inherit (v) name version default_main_file_name;
        compile_script =
          if builtins.isNull v.compile_script
          then null
          else pkgs.writeShellScript "${k}-compile.sh" v.compile_script;
        run_script = pkgs.writeShellScript "${k}-run.sh" v.run_script;
        closure = (rootPaths: "${pkgs.closureInfo {inherit rootPaths;}}/store-paths") ([run_script]
          ++ (
            if compile_script != null
            then [compile_script]
            else []
          ));
      }
      // pkgs.lib.optionalAttrs dev {inherit (v) test;})
    packages;
  environments = dev:
    pkgs.writeText "environments.json" (builtins.toJSON {
      nsjail_path = "${pkgs.nsjail}/bin/nsjail";
      time_path = "${time}/bin/time";
      environments = envs dev;
    });
}
