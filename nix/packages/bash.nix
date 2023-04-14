{
  bash,
  lib,
  ...
} @ pkgs: {
  name = "Bash";
  version = bash.version;
  compile_script = null;
  run_script = let
    path = with pkgs; [
      bashInteractive
      coreutils-full
      moreutils
      gawk
      gnused
      gnugrep
      jq
      yq
    ];
  in ''PATH='${lib.makeBinPath path}' ${bash}/bin/bash "/program/$MAIN" "$@"'';
  test.files = [
    {
      name = "test.sh";
      content = ''
        set -e
        for cmd in id ls grep awk sed jq yq xq; do
          $cmd --help > /dev/null
        done
        echo OK
      '';
    }
  ];
}
