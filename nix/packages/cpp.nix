{gcc, ...}: {
  name = "C++";
  version = gcc.version;
  compile_script = ''${gcc}/bin/g++ -std=c++17 -O2 -o /program/binary "$1"'';
  run_script = ''shift; /program/binary "$@"'';
  test.files = [
    {
      name = "test.cpp";
      content = ''
        #include "foo.cpp"

        int main(int argc, char** argv) {
          std::string s;
          std::cin >> s;
          if (s != "stdin") return 1;

          if (argc-1 != 3) return 2;
          if (strcmp(argv[1], "foo")) return 3;
          if (strcmp(argv[2], "bar")) return 4;
          if (strcmp(argv[3], "baz")) return 5;

          std::ifstream file;
          file.open("test.txt");
          getline(file, s);
          if (s != "hello world") return 6;

          bar();
          return 0;
        }
      '';
    }
    {
      name = "foo.cpp";
      content = ''
        #include <bits/stdc++.h>
        void bar() {
          std::cout << "OK";
        }
      '';
    }
  ];
}
