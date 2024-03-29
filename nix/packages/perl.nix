{pkgs, ...}: {
  name = "Perl";
  version = pkgs.perl.version;
  meta = {
    inherit (pkgs.perl.meta) homepage;
  };
  default_main_file_name = "code.pl";
  compile_script = null;
  run_script = ''${pkgs.perl}/bin/perl -I/program /program/"$@"'';
  example = ''
    my $name = <STDIN>;
    print("Hello, ''${name}!");
  '';
  test.main_file.content = ''
    use Foo;

    if (<STDIN> ne "stdin") { exit 1; }

    if ($#ARGV + 1 != 3) { exit 2; }
    if ($ARGV[0] ne "foo") { exit 3; }
    if ($ARGV[1] ne "bar") { exit 4; }
    if ($ARGV[2] ne "baz") { exit 5; }

    open(my $fh, '<', 'test.txt') or die "Can't open file: $!";
    my $file_content = do { local $/; <$fh> };
    close($fh);

    if ($file_content ne "hello world") { exit 6; }

    print($Foo::X)
  '';
  test.files = [
    {
      name = "Foo.pm";
      content = ''
        package Foo;

        our $X = "OK";

        1;
      '';
    }
  ];
}
