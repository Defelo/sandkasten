{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/release-23.05";
    nixpkgs-old.url = "github:NixOS/nixpkgs/release-22.11";
  };

  outputs = {
    self,
    nixpkgs,
    nixpkgs-old,
    ...
  }: let
    system = "x86_64-linux";
    pkgs = import nixpkgs {inherit system;};
    pkgs-old = import nixpkgs-old {inherit system;};
    lib = import ./nix/lib.nix {inherit pkgs pkgs-old;};
  in {
    packages.${system} = {
      inherit (lib) packages;
      time = pkgs.callPackage ./nix/time {};
      sandkasten = pkgs.callPackage ./nix/sandkasten.nix {};
      default = self.packages.${system}.sandkasten;
    };
    nixosModules.sandkasten = import ./nix/nixos/module.nix {
      inherit self lib;
    };
    devShells.${system} = import ./nix/dev/shell.nix {inherit self pkgs lib;};
    templates.vm.path = ./nix/nixos/vm;
  };
}
