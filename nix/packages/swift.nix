{swift, ...}: {
  name = "Swift";
  version = swift.version;
  meta = {
    inherit (swift.meta) description homepage;
  };
  default_main_file_name = "code.swift";
  compile_script = null;
  run_script = ''${swift}/bin/swift -module-cache-path /tmp /program/"$@"'';
  test.main_file.content = ''
    print("OK")
  '';
  test.files = [];
}
