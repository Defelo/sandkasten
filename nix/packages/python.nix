{python311, ...}: {
  name = "Python";
  version = python311.version;
  compile_script = null;
  run_script = ''${python311}/bin/python "/program/$MAIN" "$@"'';
}
