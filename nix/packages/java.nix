{
  jdk,
  coreutils,
  gnugrep,
  ...
}: {
  name = "Java";
  version = jdk.version;
  compile_script = ''
    set -e
    ${jdk}/bin/javac -d /out "$1"
    for file in /out/*.class; do
      cls=$(${coreutils}/bin/basename "$file" .class)
      if ${jdk}/bin/javap -public "$file" | ${gnugrep}/bin/grep -q '^  public static void main(java.lang.String\[\]);$'; then
        echo "$cls" > /out/.main
        break
      fi
    done
    if ! [[ -f /out/.main ]]; then
      echo "Could not find main class"
      exit 1
    fi
    ${jdk}/bin/javac -d /out "$@"
  '';
  run_script = ''${jdk}/bin/java -cp /program "$(${coreutils}/bin/cat /program/.main)" "$@"'';
}
