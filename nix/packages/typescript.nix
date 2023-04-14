{
  typescript,
  nodejs,
  coreutils,
  ...
}: {
  name = "TypeScript";
  version = typescript.version;
  compile_script = ''${typescript}/bin/tsc --outDir /program/ "$@"'';
  run_script = ''${nodejs}/bin/node /program/$(${coreutils}/bin/basename "$MAIN" .ts).js "$@"'';
  test.files = [
    {
      name = "test.ts";
      content = ''
        console.log("OK");
      '';
    }
  ];
}
