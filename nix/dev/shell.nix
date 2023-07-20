{
  pkgs,
  lib,
  ...
}: let
  inherit (lib) time config limits;
  packages = builtins.removeAttrs lib.packages ["all" "combined"];
  test-env = {
    PACKAGES_TEST_SRC = pkgs.writeText "packages_test_src.rs" (builtins.foldl' (acc: pkg:
      acc
      + ''
        #[test]
        #[ignore]
        fn test_${pkg}() {
          test_package("${pkg}");
        }
        #[test]
        #[ignore]
        fn example_${pkg}() {
          test_example("${pkg}");
        }
      '') "" (builtins.attrNames packages));
    ENVIRONMENTS_LIST_SRC = pkgs.writeText "environments_list_src.rs" ''
      const ENVIRONMENTS: &[&str] = &[${builtins.foldl' (acc: x: acc + ''"${x}", '') "" (builtins.attrNames packages)}];
    '';
    LIMITS_TEST_SRC = pkgs.writeText "limits_test_src.rs" (let
      numeric = builtins.mapAttrs (k: v: v.min) limits.u64;
      compile.network =
        if config.compile_limits.network
        then "any::<bool>()"
        else "Just(false)";
      run.network =
        if config.run_limits.network
        then "any::<bool>()"
        else "Just(false)";
    in ''
      prop_compose! {
          fn compile_limits() (
              ${builtins.foldl' (acc: x: acc + "${x} in option::of(${toString numeric.${x}}u64..=${toString config.compile_limits.${x}}), ") "network in option::of(${compile.network}), " (builtins.attrNames numeric)}
          ) -> LimitsOpt {
              LimitsOpt {
                network,
                ${builtins.foldl' (acc: x: acc + "${x}, ") "" (builtins.attrNames numeric)}
              }
          }
      }
      prop_compose! {
          fn run_limits() (
              ${builtins.foldl' (acc: x: acc + "${x} in option::of(${toString numeric.${x}}u64..=${toString config.run_limits.${x}}), ") "network in option::of(${run.network}), " (builtins.attrNames numeric)}
          ) -> LimitsOpt {
              LimitsOpt {
                network,
                ${builtins.foldl' (acc: x: acc + "${x}, ") "" (builtins.attrNames numeric)}
              }
          }
      }
    '');
    CONFIG_PATH = pkgs.writeText "config.json" (builtins.toJSON (config
      // {
        host = "127.0.0.1";
        port = 8000;
        server = "/";
        enable_metrics = true;
        programs_dir = "programs";
        jobs_dir = "jobs";
        program_ttl = 60;
        prune_programs_interval = 30;
        run_limits = config.run_limits // {network = true;};
        nsjail_path = ".nsjail";
        time_path = "${lib.time}/bin/time";
      }));
  };
  test-script = pkgs.writeShellScript "integration-tests.sh" ''
    export PROPTEST_CASES=''${1:-256}
    rm -rf programs jobs
    echo 'save ""' | ${pkgs.redis}/bin/redis-server - &
    redis_pid=$!
    RUST_LOG=info,poem::middleware::tracing_mw=off cargo llvm-cov run --lcov --output-path lcov-server.info --release --locked -F test_api &
    pid=$!
    while ! ${pkgs.curl}/bin/curl -so/dev/null localhost:8000; do
      sleep 1
    done
    cargo llvm-cov test --lcov --output-path lcov-tests.info --locked --all-features --all-targets --no-fail-fast -- --include-ignored
    out=$?
    ${pkgs.curl}/bin/curl -X POST localhost:8000/test/exit
    wait $pid
    kill $redis_pid
    ${pkgs.lcov}/bin/lcov -a lcov-server.info -a lcov-tests.info -o lcov.info
    ${pkgs.gnugrep}/bin/grep -E -o '^cc [0-9a-f]{64}' tests/proptests.proptest-regressions
    exit $out
  '';
  cov = pkgs.writeShellScript "cov.sh" ''
    rm -rf lcov*.info lcov_html
    ${test-script} "''${1:-16}"
    ${pkgs.lcov}/bin/genhtml -o lcov_html lcov.info
  '';
  setup-nsjail = pkgs.writeShellScript "setup-nsjail.sh" ''
    if [[ "$UID" != 0 ]]; then
      exec sudo "$0" "$@"
    fi
    cp -a ${pkgs.nsjail}/bin/nsjail .nsjail
    chmod +s .nsjail
  '';
  scripts = pkgs.stdenv.mkDerivation {
    name = "scripts";
    unpackPhase = "true";
    installPhase = ''
      mkdir -p $out/bin \
        && ln -s ${test-script} $out/bin/integration-tests \
        && ln -s ${cov} $out/bin/cov \
        && ln -s ${setup-nsjail} $out/bin/setup-nsjail
    '';
  };
in {
  default = pkgs.mkShell ({
      packages = [pkgs.cargo-llvm-cov pkgs.lcov pkgs.redis time scripts];
      RUST_LOG = "info,sandkasten=trace,difft=off";
    }
    // test-env);
  test = pkgs.mkShell ({packages = [pkgs.cargo-llvm-cov scripts];} // test-env);
}
