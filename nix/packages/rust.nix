{
  rustc,
  gcc,
  ...
}: {
  name = "Rust";
  version = rustc.version;
  compile_script = ''PATH=${gcc}/bin/ ${rustc}/bin/rustc -O -o /program/binary "$1"'';
  run_script = ''shift; /program/binary "$@"'';
  test.files = [
    {
      name = "test.rs";
      content = ''
        mod foo;
        fn main() {
          foo::bar();
        }
      '';
    }
    {
      name = "foo.rs";
      content = ''
        pub fn bar() {
          println!("OK");
        }
      '';
    }
  ];
}
