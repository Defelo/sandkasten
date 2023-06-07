{pkgs, ...}: {
  boot.loader.grub.enable = true;
  boot.loader.grub.device = "/dev/sda";
  boot.loader.timeout = 2;

  boot.tmp.useTmpfs = true;

  console.keyMap = "us";

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
  environment.shellAliases.vim = "nvim";

  system.stateVersion = "23.05";
}
