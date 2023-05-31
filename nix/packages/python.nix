{python311, ...}: let
  pkgs = p: with p; [numpy pandas scipy sympy pycrypto requests];
in {
  name = "Python";
  version = python311.version;
  meta = {
    inherit (python311.meta) description longDescription homepage;
    packages = map (p: p.pname) (pkgs python311.pkgs);
  };
  default_main_file_name = "code.py";
  compile_script = null;
  run_script = ''${python311.withPackages pkgs}/bin/python /program/"$@"'';
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
