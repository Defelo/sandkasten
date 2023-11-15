{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/release-23.05";
    nixpkgs-old.url = "github:NixOS/nixpkgs/release-22.11";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    nixpkgs-old,
    ...
  } @ inputs: let
    system = "x86_64-linux";
    pkgs = import nixpkgs {inherit system;};
    pkgs-old = import nixpkgs-old {inherit system;};
    lib = import ./nix/lib.nix {inherit pkgs pkgs-old;};
  in rec {
    packages.${system} = {
      inherit (lib) packages;
      time = pkgs.callPackage ./nix/time {};
      sandkasten = pkgs.callPackage ./nix/sandkasten.nix {inherit inputs;};
      default = self.packages.${system}.sandkasten;
    };
    nixosModules.sandkasten = import ./nix/nixos/module.nix {
      inherit lib;
      inherit (packages.${system}) default;
    };
    devShells.${system} = import ./nix/dev/shell.nix {inherit pkgs lib;};
    templates.vm.path = ./nix/nixos/vm;
  };
}
