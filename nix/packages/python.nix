{pkgs, ...}: let
  py-pkgs = p: with p; [numpy];
in {
  name = "Python";
  version = pkgs.python311.version;
  meta = {
    inherit (pkgs.python311.meta) description longDescription homepage;
    packages = map (p: p.pname) (py-pkgs pkgs.python311.pkgs);
  };
  default_main_file_name = "code.py";
  compile_script = null;
  run_script = ''${pkgs.python311.withPackages py-pkgs}/bin/python /program/"$@"'';
  example = ''
    name = input()
    print(f"Hello, {name}!")
  '';
  test.main_file.content = ''
    import sys
    import foo

    if input() != "stdin": exit(1)

    if len(sys.argv)-1 != 3: exit(2)
    if sys.argv[1] != "foo": exit(3)
    if sys.argv[2] != "bar": exit(4)
    if sys.argv[3] != "baz": exit(5)

    if open("test.txt").read() != "hello world": exit(6)

    foo.bar()
  '';
  test.files = [
    {
      name = "foo.py";
      content = ''
        def bar():
          print("OK")
      '';
    }
  ];
}
