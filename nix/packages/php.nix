{php, ...}: {
  name = "PHP";
  version = php.version;
  compile_script = null;
  run_script = ''${php}/bin/php "/program/$MAIN" "$@"'';
  test.files = [
    {
      name = "test.php";
      content = ''
        <?php echo 'OK'; ?>
      '';
    }
  ];
}
