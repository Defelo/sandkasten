{pkgs, ...}: {
  c = import ./c.nix pkgs;
  python = import ./python.nix pkgs;
  rust = import ./rust.nix pkgs;
}
