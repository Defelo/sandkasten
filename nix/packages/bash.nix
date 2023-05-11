{
  bash,
  lib,
  ...
} @ pkgs: {
  name = "Bash";
  version = bash.version;
  default_main_file_name = "code.sh";
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
  in ''PATH='/program/:${lib.makeBinPath path}' ${bash}/bin/bash /program/"$@"'';
  test.main_file.content = ''
    set -e

    [[ "$(cat)" = "stdin" ]] || exit 1

    [[ $# -eq 3 ]] || exit 2
    [[ "$1" = "foo" ]] || exit 3
    [[ "$2" = "bar" ]] || exit 4
    [[ "$3" = "baz" ]] || exit 5

    [[ "$(cat test.txt)" = "hello world" ]] || exit 6

    for cmd in id ls grep awk sed jq yq xq; do
      $cmd --help > /dev/null
    done

    . foo.sh
    bar
  '';
  test.files = [
    {
      name = "foo.sh";
      content = ''
        bar() {
          echo OK
        }
      '';
    }
  ];
}
