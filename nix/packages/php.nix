{php, ...}: {
  name = "PHP";
  version = php.version;
  compile_script = null;
  run_script = ''${php}/bin/php /program/"$@"'';
  test.files = [
    {
      name = "test.php";
      content = ''
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
    }
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
