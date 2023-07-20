{pkgs-old, ...}: {
  name = "Swift";
  version = pkgs-old.swift.version;
  meta = {
    inherit (pkgs-old.swift.meta) description homepage;
  };
  default_main_file_name = "code.swift";
  compile_script = null;
  run_script = ''${pkgs-old.swift}/bin/swift -module-cache-path /tmp /program/"$@"'';
  example = ''
    let name = readLine()!
    print("Hello, " + name + "!")
  '';
  test.main_file.content = ''
    print("OK")
  '';
  test.files = [];
}
