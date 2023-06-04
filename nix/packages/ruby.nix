{pkgs, ...}: let
  ruby = pkgs.ruby_3_1;
in {
  name = "Ruby";
  version = toString ruby.version;
  meta = {
    inherit (ruby.meta) description homepage;
  };
  default_main_file_name = "code.rb";
  compile_script = null;
  run_script = ''${ruby}/bin/ruby -I/program /program/"$@"'';
  test.main_file.content = ''
    require 'foo.rb'

    if $stdin.readline.chomp != "stdin" then exit(1) end

    if ARGV.length != 3 then exit(2) end
    if ARGV[0] != "foo" then exit(3) end
    if ARGV[1] != "bar" then exit(4) end
    if ARGV[2] != "baz" then exit(5) end

    file_content = File.read("test.txt").strip
    if file_content != "hello world" then exit(6) end

    Foo::bar
  '';
  test.files = [
    {
      name = "foo.rb";
      content = ''
        module Foo
          def self.bar
            puts 'OK'
          end
        end
      '';
    }
  ];
}
