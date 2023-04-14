{lua5_4, ...}: let
  lua = lua5_4;
in {
  name = "Lua";
  version = lua.version;
  compile_script = null;
  run_script = ''${lua}/bin/lua "/program/$MAIN" "$@"'';
  test.files = [
    {
      name = "test.lua";
      content = ''
        print("OK")
      '';
    }
  ];
}
