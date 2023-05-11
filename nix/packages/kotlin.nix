{
  kotlin,
  coreutils,
  ...
}: {
  name = "Kotlin";
  version = kotlin.version;
  default_main_file_name = "code.kt";
  compile_script = ''PATH=${coreutils}/bin ${kotlin}/bin/kotlinc -d /program/program.jar "$@"'';
  run_script = ''shift; PATH=${coreutils}/bin ${kotlin}/bin/kotlin /program/program.jar "$@"'';
  test.main_file.content = ''
    import foo.bar;

    fun main() {
        bar()
    }
  '';
  test.files = [
    {
      name = "foo.kt";
      content = ''
        package foo;

        fun bar() {
          println("OK")
        }
      '';
    }
  ];
}
