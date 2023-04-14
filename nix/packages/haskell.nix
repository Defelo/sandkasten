{
  ghc,
  coreutils,
  gcc,
  ...
}: {
  name = "Haskell";
  version = ghc.version;
  compile_script = ''
    ${coreutils}/bin/cp $(${coreutils}/bin/ls -A) /tmp
    cd /tmp
    PATH=${gcc}/bin ${ghc}/bin/ghc -O -o /program/binary "$1"
  '';
  run_script = ''/program/binary "$@"'';
  test.files = [
    {
      name = "test.hs";
      content = ''
        main = putStrLn "OK"
      '';
    }
  ];
}
