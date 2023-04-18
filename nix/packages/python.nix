{python311, ...}: {
  name = "Python";
  version = python311.version;
  compile_script = null;
  run_script = ''${python311}/bin/python /program/"$@"'';
  test.files = [
    {
      name = "test.py";
      content = ''
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
    }
    {
      name = "foo.py";
      content = ''
        def bar():
          print("OK")
      '';
    }
  ];
}
