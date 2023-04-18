{nodejs, ...}: {
  name = "JavaScript";
  version = nodejs.version;
  compile_script = null;
  run_script = ''${nodejs}/bin/node /program/"$@"'';
  test.files = [
    {
      name = "test.js";
      content = ''
        let fs = require("fs");
        let foo = require("./foo.js");

        if (fs.readFileSync(0).toString() != "stdin") process.exit(1);

        if (process.argv.length-2 != 3) process.exit(2);
        if (process.argv[2] != "foo") process.exit(3);
        if (process.argv[3] != "bar") process.exit(4);
        if (process.argv[4] != "baz") process.exit(5);

        if (fs.readFileSync("test.txt").toString() != "hello world") process.exit(6);

        foo.bar();
      '';
    }
    {
      name = "foo.js";
      content = ''
        function bar() {
          console.log("OK");
        }
        module.exports = { bar };
      '';
    }
  ];
}
