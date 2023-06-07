{sandkasten, ...}: {
  imports = [sandkasten.nixosModules.sandkasten];

  services.sandkasten = {
    enable = true;

    environments = p: with p; [all];

    host = "0.0.0.0";
    port = 80;

    program_ttl = 300;
    prune_programs_interval = 60;

    max_concurrent_jobs = 4;

    compile_limits = {
      time = 30;
      memory = 1024;
      network = false;
    };
    run_limits = {
      time = 20;
      memory = 1024;
      network = false;
    };
  };
}
