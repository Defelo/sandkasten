{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/release-24.05";
    nixpkgs-old.url = "github:NixOS/nixpkgs/release-22.11";
    nixpkgs-master.url = "github:NixOS/nixpkgs";
    fenix.url = "github:nix-community/fenix";
    crate2nix.url = "github:nix-community/crate2nix";
  };

  outputs = {
    self,
    nixpkgs,
    fenix,
    crate2nix,
    ...
  } @ inputs: let
    defaultSystems = [
      "x86_64-linux"
      "x86_64-darwin"
      "aarch64-linux"
      "aarch64-darwin"
    ];
    eachDefaultSystem = f:
      builtins.listToAttrs (map (system: {
          name = system;
          value = f (import nixpkgs {inherit system;});
        })
        defaultSystems);
    lib = import ./nix/lib.nix;
  in {
    packages = eachDefaultSystem (pkgs: {
      default = self.packages.${pkgs.system}.sandkasten;
      sandkasten = pkgs.callPackage ./nix/sandkasten.nix {inherit fenix crate2nix;};
      packages = import ./nix/packages {
        inherit inputs lib;
        inherit (pkgs) system;
      };
      time = pkgs.callPackage ./nix/time {};
    });
    nixosModules.sandkasten = import ./nix/nixos/module.nix {
      inherit self lib;
    };
    devShells = eachDefaultSystem (pkgs: import ./nix/dev/shell.nix {inherit self pkgs lib;});
    templates.vm.path = ./nix/nixos/vm;
  };
}
