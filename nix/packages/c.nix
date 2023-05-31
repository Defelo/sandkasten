{gcc, ...}: {
  name = "C";
  version = "17";
  meta = {
    compiler = {
      name = "GCC";
      version = gcc.version;
      inherit (gcc.meta) description longDescription homepage;
    };
  };
  default_main_file_name = "code.c";
  compile_script = ''${gcc}/bin/gcc -std=c17 -O2 -o /program/binary "$1"'';
  run_script = ''shift; /program/binary "$@"'';
  test.main_file.content = ''
    #define  _GNU_SOURCE
    #include <stdlib.h>
    #include <string.h>
    #include "foo.c"

    int main(int argc, char** argv) {
      size_t bufsize = 5;
      char* buffer = (char *)malloc(bufsize * sizeof(char));
      size_t characters = getline(&buffer,&bufsize,stdin);
      if (strcmp(buffer, "stdin")) return 1;

      if (argc-1 != 3) return 2;
      if (strcmp(argv[1], "foo")) return 3;
      if (strcmp(argv[2], "bar")) return 4;
      if (strcmp(argv[3], "baz")) return 5;

      FILE* fptr = fopen("test.txt", "r");
      char content[12];
      fgets(content, 12, fptr);
      if (strcmp(content, "hello world")) return 6;

      bar();
      return 0;
    }
  '';
  test.files = [
    {
      name = "foo.c";
      content = ''
        #include <stdio.h>
        void bar() {
          printf("OK");
        }
      '';
    }
  ];
}
