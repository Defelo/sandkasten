{pkgs, ...}: {
  c = import ./c.nix pkgs;
  cpp = import ./cpp.nix pkgs;
  python = import ./python.nix pkgs;
  rust = import ./rust.nix pkgs;
}
