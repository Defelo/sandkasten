{ruby_3_1, ...}: let
  ruby = ruby_3_1;
in {
  name = "Ruby";
  version = toString ruby.version;
  compile_script = null;
  run_script = ''${ruby}/bin/ruby "/program/$MAIN" "$@"'';
  test.files = [
    {
      name = "test.rb";
      content = ''
        puts("OK")
      '';
    }
  ];
}
