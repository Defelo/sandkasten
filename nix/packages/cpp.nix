{gcc, ...}: {
  name = "C++";
  version = gcc.version;
  compile_script = ''${gcc}/bin/g++ -std=c++17 -O2 -o /out/binary "$1"'';
  run_script = ''/program/binary "$@"'';
  test.files = [
    {
      name = "test.cpp";
      content = ''
        #include <bits/stdc++.h>
        int main() {
          std::cout << "OK";
        }
      '';
    }
  ];
}
