{pkgs, ...}: {
  time = import ./time pkgs;
  cargotoml = builtins.fromTOML (builtins.readFile ../Cargo.toml);
  config = builtins.fromTOML (builtins.readFile ../config.toml);
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
}
