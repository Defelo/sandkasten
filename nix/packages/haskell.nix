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
    PATH=${gcc}/bin ${ghc}/bin/ghc -O -o /program/binary --make "$1"
  '';
  run_script = ''shift; /program/binary "$@"'';
  test.files = [
    {
      name = "test.hs";
      content = ''
        import System.Environment
        import System.Exit
        import Foo

        main :: IO ()
        main = do
          inputStr <- getLine
          if inputStr /= "stdin"
            then exitWith (ExitFailure 1)
            else do
              args <- getArgs
              if length args /= 3
                then exitWith (ExitFailure 2)
                else do
                  let arg1 = args !! 0
                      arg2 = args !! 1
                      arg3 = args !! 2
                  if arg1 /= "foo"
                    then exitWith (ExitFailure 3)
                    else if arg2 /= "bar"
                      then exitWith (ExitFailure 4)
                      else do
                        fileContent <- readFile "test.txt"
                        if fileContent /= "hello world"
                          then exitWith (ExitFailure 6)
                          else do
                            putStrLn bar
      '';
    }
    {
      name = "Foo.hs";
      content = ''
        module Foo where
        bar = "OK"
      '';
    }
  ];
}
