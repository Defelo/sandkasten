{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/release-22.11";
  };

  outputs = {
    # self,
    nixpkgs,
    ...
  }: let
    system = "x86_64-linux";
    pkgs = import nixpkgs {inherit system;};
    # packages = {
    #   python311 = {
    #     package = pkgs.python311;
    #     run = "python";
    #   };
    #   python39 = {
    #     package = pkgs.python39;
    #     run = "python";
    #   };
    # };
    packages = {
      python = {
        name = "Python";
        version = pkgs.python311.version;
        compile_script = null;
        run_script = ''${pkgs.python311}/bin/python "$@"'';
      };
      rust = {
        name = "Rust";
        version = pkgs.rustc.version;
        compile_script = ''PATH=${pkgs.gcc}/bin/ ${pkgs.rustc}/bin/rustc -O -o /out/binary "$1"'';
        run_script = ''shift; ./binary "$@"'';
      };
    };
    environments = pkgs.writeText "environments.json" (builtins.toJSON {
      nsjail_path = "${pkgs.nsjail}/bin/nsjail";
      environments =
        builtins.mapAttrs (k: v: {
          inherit (v) name version;
          compile_script =
            if builtins.isNull v.compile_script
            then null
            else pkgs.writeShellScript "${k}-compile.sh" v.compile_script;
          run_script = pkgs.writeShellScript "${k}-run.sh" v.run_script;
        })
        packages;
    });
  in {
    packages.${system}.default = pkgs.dockerTools.buildLayeredImage {
      name = "sandkasten";
      tag = "latest";
      contents = with pkgs; [
        nsjail
        coreutils-full
        # (pkgs.stdenv.mkDerivation {
        #   name = "packages";
        #   src = self;
        #   nativeBuildInputs = [pkgs.makeWrapper];
        #   installPhase =
        #     "mkdir -p $out/bin"
        #     + builtins.foldl' (acc: k: let
        #       script = pkgs.writeShellScript "${k}-run.sh" packages.${k}.run;
        #     in
        #       acc + " && cp ${script} $out/bin/${k}-run.sh") "" (builtins.attrNames packages);
        #   postFixup = builtins.foldl' (acc: k: acc + " && wrapProgram $out/bin/${k}-run.sh --set PATH ${pkgs.lib.makeBinPath [packages.${k}.package]}") "true" (builtins.attrNames packages);
        # })
      ];
      config = {
        User = "65534:65534";
        Entrypoint = ["${pkgs.bash}/bin/bash"];
      };
    };
    devShells.${system}.default = pkgs.mkShell {
      buildInputs = [pkgs.nsjail];
      CONFIG_PATH = ".config.toml";
      ENVIRONMENTS_CONFIG_PATH = environments;
    };
  };
}
