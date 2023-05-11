{
  dotnet-sdk,
  coreutils,
  ...
}: {
  name = "C#";
  version = dotnet-sdk.version;
  default_main_file_name = "code.cs";
  compile_script = ''
    ${coreutils}/bin/cp $(${coreutils}/bin/ls -A) /tmp
    export HOME=/tmp/.dotnet
    export DOTNET_CLI_TELEMETRY_OPTOUT=1
    cd /tmp
    ${dotnet-sdk}/bin/dotnet new console -o . --no-restore
    ${coreutils}/bin/rm Program.cs
    ${dotnet-sdk}/bin/dotnet restore
    ${dotnet-sdk}/bin/dotnet build --no-restore -o /program
  '';
  run_script = ''
    shift
    export HOME=/tmp/.dotnet
    export DOTNET_CLI_TELEMETRY_OPTOUT=1
    ${dotnet-sdk}/bin/dotnet /program/*.dll "$@"
  '';
  test.main_file.content = ''
    using System;

    public class Test {
        public static int Main(string[] args) {
            string s = Console.In.ReadLine();
            if (s != "stdin") return 1;

            if (args.Length != 3) return 2;
            if (args[0] != "foo") return 3;
            if (args[1] != "bar") return 4;
            if (args[2] != "baz") return 5;

            s = File.ReadAllText("test.txt");
            if (s != "hello world") return 6;

            Foo.bar();
            return 0;
        }
    }
  '';
  test.files = [
    {
      name = "foo.cs";
      content = ''
        using System;

        public class Foo {
          public static void bar() {
            Console.WriteLine("OK");
          }
        }
      '';
    }
  ];
}
