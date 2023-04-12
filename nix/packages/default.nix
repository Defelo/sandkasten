{pkgs, ...}: {
  c = import ./c.nix pkgs;
  cpp = import ./cpp.nix pkgs;
  javascript = import ./javascript.nix pkgs;
  python = import ./python.nix pkgs;
  rust = import ./rust.nix pkgs;
  typescript = import ./typescript.nix pkgs;
}
