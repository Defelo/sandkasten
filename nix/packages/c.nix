{gcc, ...}: {
  name = "C";
  version = gcc.version;
  compile_script = ''${gcc}/bin/gcc -std=c11 -O2 -o /out/binary "$1"'';
  run_script = ''/program/binary "$@"'';
}
