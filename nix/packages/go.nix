{pkgs, ...}: {
  name = "Go";
  version = pkgs.go.version;
  meta = {
    inherit (pkgs.go.meta) description homepage;
  };
  default_main_file_name = "code.go";
  compile_script = ''HOME=/tmp ${pkgs.go}/bin/go build -o /program/binary "$1"'';
  run_script = ''shift; /program/binary "$@"'';
  example = ''
    package main

    import "fmt"

    func main() {
      var name string
      fmt.Scanln(&name)
      fmt.Printf("Hello, %s!\n", name)
    }
  '';
  test.main_file.content = ''
    package main

    import "fmt"
    import "os"

    func main() {
      var stdin string
      fmt.Scanln(&stdin)
      if stdin != "stdin" { os.Exit(1) }

      if len(os.Args) != 4 || os.Args[1] != "foo" || os.Args[2] != "bar" || os.Args[3] != "baz" {
        os.Exit(2)
      }

      dat, err := os.ReadFile("test.txt")
      if err != nil || string(dat) != "hello world" { os.Exit(3) }

      fmt.Println("OK")
    }
  '';
  # TODO: test imports
  test.files = [];
}
