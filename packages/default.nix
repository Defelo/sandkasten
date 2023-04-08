{pkgs, ...}: {
  python = import ./python.nix pkgs;
  rust = import ./rust.nix pkgs;
}
