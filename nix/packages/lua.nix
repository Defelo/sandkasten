{lua5_4, ...}: let
  lua = lua5_4;
in {
  name = "Lua";
  version = lua.version;
  compile_script = null;
  run_script = ''LUA_PATH=/program/?.lua ${lua}/bin/lua /program/"$@"'';
  test.files = [
    {
      name = "test.lua";
      content = ''
        require "foo"

        if io.read() ~= "stdin" then os.exit(1) end

        if #arg ~= 3 then os.exit(2) end
        if arg[1] ~= "foo" then os.exit(3) end
        if arg[2] ~= "bar" then os.exit(4) end
        if arg[3] ~= "baz" then os.exit(5) end

        local file = io.open("test.txt", "r")
        if file:read("*all") ~= "hello world" then
            file:close()
            os.exit(6)
        end
        file:close()

        bar()
      '';
    }
    {
      name = "foo.lua";
      content = ''
        function bar()
          print("OK")
        end
      '';
    }
  ];
}
