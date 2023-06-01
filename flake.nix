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
    nixpkgs,
    nixpkgs-old,
    fenix,
    naersk,
    ...
  }: let
    system = "x86_64-linux";
    pkgs = import nixpkgs {inherit system;};
    pkgs-old = import nixpkgs-old {inherit system;};
    lib = import ./nix/lib.nix {inherit pkgs pkgs-old;};
  in rec {
    packages.${system} = rec {
      packages = import ./nix/packages {inherit pkgs pkgs-old;};
      rust = import ./nix/rust.nix {inherit system pkgs fenix naersk;};
      docker = import ./nix/docker.nix {inherit pkgs lib rust;};
      default = import ./nix/default.nix {inherit pkgs lib rust;};
    };
    nixosModules.sandkasten = import ./nix/nixos.nix {
      inherit lib;
      inherit (packages.${system}) default;
    };
    devShells.${system} = import ./nix/dev/shell.nix {
      inherit pkgs lib;
      packages = builtins.removeAttrs packages.${system}.packages ["all"];
    };
  };
}
