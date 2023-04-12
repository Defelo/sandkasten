{
  typescript,
  nodejs,
  coreutils,
  ...
}: {
  name = "TypeScript";
  version = typescript.version;
  compile_script = ''${typescript}/bin/tsc --outDir /out/ "$@"'';
  run_script = ''${nodejs}/bin/node /program/$(${coreutils}/bin/basename "$MAIN" .ts).js "$@"'';
}
