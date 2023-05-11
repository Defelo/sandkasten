{
  typescript,
  nodejs,
  coreutils,
  stdenv,
  fetchurl,
  ...
}: let
  node_modules = stdenv.mkDerivation {
    name = "node_modules";
    src = fetchurl {
      url = "https://registry.npmjs.org/@types/node/-/node-18.15.11.tgz";
      sha512 = "E5Kwq2n4SbMzQOn6wnmBjuK9ouqlURrcZDVfbo9ftDDTFt3nk7ZKK4GMOzoYgnpQJKcxwQw+lGaBvvlMo0qN/Q==";
    };
    installPhase = "mkdir -p $out/node_modules/@types/node; mv $(ls -A) $out/node_modules/@types/node/";
  };
in {
  name = "TypeScript";
  version = typescript.version;
  default_main_file_name = "code.ts";
  compile_script = ''
    ${coreutils}/bin/cp $(${coreutils}/bin/ls -A) /tmp
    cd /tmp
    ${coreutils}/bin/ln -s ${node_modules}/node_modules .
    ${typescript}/bin/tsc -m node16 --outDir /program/ "$@"
  '';
  run_script = ''
    main=/program/$(${coreutils}/bin/basename "$1" .ts).js
    shift
    ${nodejs}/bin/node "$main" "$@"
  '';
  test.main_file.content = ''
    import { bar } from "./foo";
    import * as fs from "fs";

    if (fs.readFileSync(0).toString() != "stdin") process.exit(1);

    if (process.argv.length-2 != 3) process.exit(2);
    if (process.argv[2] != "foo") process.exit(3);
    if (process.argv[3] != "bar") process.exit(4);
    if (process.argv[4] != "baz") process.exit(5);

    if (fs.readFileSync("test.txt").toString() != "hello world") process.exit(6);

    bar()
  '';
  test.files = [
    {
      name = "foo.ts";
      content = ''
        export function bar() {
          console.log("OK")
        }
      '';
    }
  ];
}
