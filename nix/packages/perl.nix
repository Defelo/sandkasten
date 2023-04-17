{perl, ...}: {
  name = "Perl";
  version = perl.version;
  compile_script = null;
  run_script = ''${perl}/bin/perl /program/"$@"'';
  test.files = [
    {
      name = "test.pl";
      content = ''
        print("OK")
      '';
    }
  ];
}
