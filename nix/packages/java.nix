{
  jdk,
  coreutils,
  gnugrep,
  gawk,
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
  run_script = ''
    mem=$(${gnugrep}/bin/grep 'address space' /proc/self/limits | ${gawk}/bin/awk '{print $5}')
    mem=$((mem/128))
    ${jdk}/bin/java -Xms$mem -Xmx$mem -cp /program "$(${coreutils}/bin/cat /program/.main)" "$@"
  '';
  test.files = [
    {
      name = "test.java";
      content = ''
        class FooBar {
          public static void main(String[] args) {
            System.out.print("OK");
          }
        }
      '';
    }
  ];
}
