{pkgs, ...}: {
  name = "Kotlin";
  version = pkgs.kotlin.version;
  meta = {
    inherit (pkgs.kotlin.meta) description longDescription homepage;
  };
  default_main_file_name = "code.kt";
  compile_script = ''PATH=${pkgs.coreutils}/bin ${pkgs.kotlin}/bin/kotlinc -d /program/program.jar "$@"'';
  run_script = ''shift; PATH=${pkgs.coreutils}/bin ${pkgs.kotlin}/bin/kotlin /program/program.jar "$@"'';
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
