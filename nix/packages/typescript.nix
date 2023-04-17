{
  typescript,
  nodejs,
  coreutils,
  ...
}: {
  name = "TypeScript";
  version = typescript.version;
  compile_script = ''${typescript}/bin/tsc --outDir /program/ "$@"'';
  run_script = ''
    main=/program/$(${coreutils}/bin/basename "$1" .ts).js
    shift
    ${nodejs}/bin/node "$main" "$@"
  '';
  test.files = [
    {
      name = "test.ts";
      content = ''
        console.log("OK");
      '';
    }
  ];
}
