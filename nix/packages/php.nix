{pkgs, ...}: {
  name = "PHP";
  version = pkgs.php.version;
  meta = {
    inherit (pkgs.php.meta) description homepage;
  };
  default_main_file_name = "code.php";
  compile_script = null;
  run_script = ''${pkgs.php}/bin/php /program/"$@"'';
  example = ''
    <?php

    $name = fgets(STDIN);
    echo "Hello, " . $name . "!";

    ?>
  '';
  test.main_file.content = ''
    <?php

    require 'foo.php';

    if (fgets(STDIN) !== "stdin") { exit(1); }

    if ($argc - 1 !== 3) { exit(2); }
    if ($argv[1] !== "foo") { exit(3); }
    if ($argv[2] !== "bar") { exit(4); }
    if ($argv[3] !== "baz") { exit(5); }

    $file_content = file_get_contents("test.txt");
    if ($file_content !== "hello world") { exit(6); }

    echo $x;
  '';
  test.files = [
    {
      name = "foo.php";
      content = ''
        <?php

        $x = 'OK';

        ?>
      '';
    }
  ];
}
