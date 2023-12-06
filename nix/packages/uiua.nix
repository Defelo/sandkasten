{pkgs-master, ...}: let
  uiua = pkgs-master.uiua;
in {
  name = "Uiua";
  version = uiua.version;
  meta = {
    inherit (uiua.meta) description longDescription homepage;
  };
  default_main_file_name = "code.ua";
  compile_script = null;
  run_script = ''${uiua}/bin/uiua run --no-format /program/"$@"'';
  example = ''
    &p $"Hello, _!" &rs 32 0
  '';
  test.main_file.content = ''
    ⍤"stdin" ≍"stdin" &fras "/proc/self/fd/0"
    ⍤"args" ≍{"foo" "bar" "baz"} ↘1 &args
    ⍤"file" ≍"hello world" &fras "test.txt"
    &p &i "foo.ua" "Bar"
  '';
  test.files = [
    {
      name = "foo.ua";
      content = ''
        Bar ← "OK"
      '';
    }
  ];
}
