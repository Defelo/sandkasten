{
  rustc,
  gcc,
  ...
}: {
  name = "Rust";
  version = rustc.version;
  meta = {
    inherit (rustc.meta) description homepage;
  };
  default_main_file_name = "code.rs";
  compile_script = ''PATH=${gcc}/bin/ ${rustc}/bin/rustc -O -o /program/binary "$1"'';
  run_script = ''shift; /program/binary "$@"'';
  test.main_file.content = ''
    mod foo;
    fn main() {
      let mut s = String::new();
      std::io::stdin().read_line(&mut s).unwrap();
      assert_eq!(s, "stdin");

      let mut args = std::env::args();
      args.next().unwrap();
      assert_eq!(args.next().unwrap(), "foo");
      assert_eq!(args.next().unwrap(), "bar");
      assert_eq!(args.next().unwrap(), "baz");

      let s = std::fs::read_to_string("test.txt").unwrap();
      assert_eq!(s, "hello world");

      foo::bar();
    }
  '';
  test.files = [
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
