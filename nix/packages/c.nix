{gcc, ...}: {
  name = "C";
  version = gcc.version;
  compile_script = ''${gcc}/bin/gcc -std=c11 -O2 -o /program/binary "$1"'';
  run_script = ''shift; /program/binary "$@"'';
  test.files = [
    {
      name = "test.c";
      content = ''
        #include <stdio.h>
        int main() {
          printf("OK");
          return 0;
        }
      '';
    }
  ];
}
