{pkgs, ...}: {
  bash = import ./bash.nix pkgs;
  c = import ./c.nix pkgs;
  cpp = import ./cpp.nix pkgs;
  java = import ./java.nix pkgs;
  javascript = import ./javascript.nix pkgs;
  lua = import ./lua.nix pkgs;
  perl = import ./perl.nix pkgs;
  php = import ./php.nix pkgs;
  python = import ./python.nix pkgs;
  ruby = import ./ruby.nix pkgs;
  rust = import ./rust.nix pkgs;
  typescript = import ./typescript.nix pkgs;
}
