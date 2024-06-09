{pkgs-master, ...}: let
  inherit (pkgs-master) uiua;
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
    &p $"Hello, _!" &fras "/dev/stdin"
  '';
  test.main_file.content = ''
    ~ "foo.ua" ~ Bar
    ⍤"stdin" ≍"stdin" &fras "/dev/stdin"
    ⍤"args" ≍{"foo" "bar" "baz"} ↘1 &args
    ⍤"file" ≍"hello world" &fras "test.txt"
    &p Bar @K
  '';
  test.files = [
    {
      name = "foo.ua";
      content = ''
        Bar ← ⊂@O
      '';
    }
  ];
}
