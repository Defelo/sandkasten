{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-23.05";
    sandkasten.url = "github:Defelo/sandkasten/latest";
  };

  outputs = {
    nixpkgs,
    sandkasten,
    ...
  }: let
    system = "x86_64-linux";
  in {
    nixosConfigurations.sandkasten = nixpkgs.lib.nixosSystem {
      inherit system;
      specialArgs = {inherit sandkasten;};
      modules = [
        ./sandkasten.nix
        ./configuration.nix
        ./hardware-configuration.nix
      ];
    };
  };
}
