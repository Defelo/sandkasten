{pkgs, ...}: {
  name = "Java";
  version = pkgs.jdk.version;
  meta = {
    inherit (pkgs.jdk.meta) homepage;
  };
  default_main_file_name = "code.java";
  compile_script = ''
    set -e
    ${pkgs.jdk}/bin/javac -d /program "$1"
    for file in /program/*.class; do
      cls=$(${pkgs.coreutils}/bin/basename "$file" .class)
      if ${pkgs.jdk}/bin/javap -public "$file" | ${pkgs.gnugrep}/bin/grep -q '^  public static void main(java.lang.String\[\])'; then
        echo "$cls" > /program/.main
        break
      fi
    done
    if ! [[ -f /program/.main ]]; then
      echo "Could not find main class"
      exit 1
    fi
    ${pkgs.jdk}/bin/javac -d /program "$@"
  '';
  run_script = ''
    shift
    mem=$(${pkgs.gnugrep}/bin/grep 'address space' /proc/self/limits | ${pkgs.gawk}/bin/awk '{print $5}')
    mem=$((mem/128))
    ${pkgs.jdk}/bin/java -Xms$mem -Xmx$mem -cp /program "$(${pkgs.coreutils}/bin/cat /program/.main)" "$@"
  '';
  test.main_file.content = ''
    import java.io.IOException;
    import java.util.Scanner;
    import java.nio.file.Files;
    import java.nio.file.Paths;

    class FooBar {
      public static void main(String[] args) throws IOException {
        Scanner s = new Scanner(System.in);
        if (!s.next().equals("stdin")) System.exit(1);

        if (args.length != 3) System.exit(2);
        if (!args[0].equals("foo")) System.exit(3);
        if (!args[1].equals("bar")) System.exit(4);
        if (!args[2].equals("baz")) System.exit(5);

        String content = Files.readString(Paths.get("test.txt"));
        if (!content.equals("hello world")) System.exit(6);

        Foo.bar();
      }
    }
  '';
  test.files = [
    {
      name = "Foo.java";
      content = ''
        public class Foo {
          public static void bar() {
            System.out.print("OK");
          }
        }
      '';
    }
  ];
}
