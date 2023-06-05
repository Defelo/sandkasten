{
  lib,
  modulesPath,
  pkgs,
  ...
}: {
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

  # === system configuration ===
  boot.loader.grub.enable = true;
  boot.loader.grub.device = "/dev/sda";
  boot.loader.timeout = 2;

  boot.tmp.useTmpfs = true;

  time.timeZone = "UTC";

  networking.hostName = "sandkasten";
  networking.firewall.allowedTCPPorts = [80];

  users.users.root.initialPassword = "sandkasten";

  services.openssh = {
    enable = true;
    settings.PermitRootLogin = "yes";
  };

  services.getty.autologinUser = "root";

  nix = {
    gc = {
      automatic = true;
      dates = "daily";
      options = "--delete-older-than 3d";
    };
    settings = {
      auto-optimise-store = true;
      experimental-features = ["nix-command" "flakes" "repl-flake"];
      substituters = [
        "https://sandkasten.cachix.org"
      ];
      trusted-public-keys = [
        "sandkasten.cachix.org-1:Pa7qfdlx7bZkko+ojaaEG9pyziZkaru9v4TfcioqNZw="
      ];
    };
  };

  environment.systemPackages = with pkgs; [
    neovim
    wget
    htop
    duf
    ncdu
    dig
    git
  ];

  system.stateVersion = "23.05";

  # === hardware configuration ===
  imports = [
    (modulesPath + "/profiles/qemu-guest.nix")
  ];

  boot.initrd.availableKernelModules = ["ata_piix" "uhci_hcd" "virtio_pci" "virtio_scsi" "sd_mod" "sr_mod"];
  boot.initrd.kernelModules = ["dm-snapshot"];
  boot.kernelModules = [];
  boot.extraModulePackages = [];

  fileSystems."/" = {
    device = "/dev/sda1";
    fsType = "ext4";
  };

  swapDevices = [];

  networking.useDHCP = lib.mkDefault true;

  nixpkgs.hostPlatform = lib.mkDefault "x86_64-linux";
}
