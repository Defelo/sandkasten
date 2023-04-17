{
  kotlin,
  coreutils,
  ...
}: {
  name = "Kotlin";
  version = kotlin.version;
  compile_script = ''PATH=${coreutils}/bin ${kotlin}/bin/kotlinc -d /program/program.jar "$@"'';
  run_script = ''shift; PATH=${coreutils}/bin ${kotlin}/bin/kotlin /program/program.jar "$@"'';
  test.files = [
    {
      name = "test.kt";
      content = ''
        fun main() {
            println("OK")
        }
      '';
    }
  ];
}
