{
  dotnet-sdk,
  coreutils,
  findutils,
  ...
}: {
  name = "C#";
  version = dotnet-sdk.version;
  compile_script = ''
    ${coreutils}/bin/cp $(${coreutils}/bin/ls -A) /tmp
    export HOME=/tmp/.dotnet
    export DOTNET_CLI_TELEMETRY_OPTOUT=1
    cd /tmp
    ${dotnet-sdk}/bin/dotnet new console -o . --no-restore
    ${coreutils}/bin/rm Program.cs
    ${dotnet-sdk}/bin/dotnet restore
    ${dotnet-sdk}/bin/dotnet build --no-restore -o /out
  '';
  run_script = ''
    export HOME=/tmp/.dotnet
    export DOTNET_CLI_TELEMETRY_OPTOUT=1
    ${dotnet-sdk}/bin/dotnet /program/*.dll "$@"
  '';
  test.files = [
    {
      name = "test.cs";
      content = ''
        using System;

        public class Test {
            public static void Main(string[] args) {
                Console.WriteLine("OK");
            }
        }
      '';
    }
  ];
}
