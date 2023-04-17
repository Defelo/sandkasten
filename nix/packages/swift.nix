{swift, ...}: {
  name = "Swift";
  version = swift.version;
  compile_script = null;
  run_script = ''${swift}/bin/swift -module-cache-path /tmp /program/"$@"'';
  test.files = [
    {
      name = "test.swift";
      content = ''
        print("OK")
      '';
    }
  ];
}
