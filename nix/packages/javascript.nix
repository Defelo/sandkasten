{nodejs, ...}: {
  name = "JavaScript";
  version = nodejs.version;
  compile_script = null;
  run_script = ''${nodejs}/bin/node "/program/$MAIN" "$@"'';
  test.files = [
    {
      name = "test.js";
      content = ''
        console.log("OK");
      '';
    }
  ];
}
