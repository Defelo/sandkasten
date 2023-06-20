{pkgs, ...}: {
  name = "SQL";
  version = pkgs.sqlite.version;
  meta = {
    inherit (pkgs.sqlite.meta) description homepage;
    dialect = "sqlite";
  };
  default_main_file_name = "code.sql";
  compile_script = null;
  run_script = ''
    export HOME=/tmp
    cd /program
    ${pkgs.sqlite}/bin/sqlite3 -json :memory: < $1
  '';
  test.main_file.content = "SELECT 'OK' as out;";
  test.files = [];
  test.expected = builtins.toJSON [{out = "OK";}];
}
