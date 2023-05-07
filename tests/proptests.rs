#![cfg(feature = "nix")]

use std::collections::HashSet;

use indoc::formatdoc;
use once_cell::unsync::Lazy;
use proptest::{collection, option, prelude::*, string::string_regex};
use regex::Regex;
use sandkasten_client::schemas::programs::{
    BuildRequest, BuildRunRequest, File, LimitsOpt, RunRequest,
};

use common::client;

mod common;

proptest! {
    #[test]
    #[ignore]
    fn test_files(build_files in files(1, 10, 256), run_files in files(0, 10, 256)) {
        client().build_and_run(&BuildRunRequest {
            build: BuildRequest {
                environment: "python".into(),
                files: build_files,
                compile_limits: Default::default(),
            },
            run: RunRequest{files: run_files, ..Default::default()},
        }).unwrap();
    }
}

proptest! {
    #[test]
    #[ignore]
    fn test_compile_limits(compile_limits in compile_limits()) {
        client().build_and_run(&BuildRunRequest {
            build: BuildRequest {
                environment: "rust".into(),
                files: vec![File {
                    name: "test.rs".into(),
                    content: formatdoc! {r#"
                        fn main() {{
                            // {compile_limits:?}
                            println!("Hello World!");
                        }}
                    "#}
                }],
                compile_limits,
            },
            run: Default::default(),
        }).ok();
    }
}

proptest! {
    #[test]
    #[ignore]
    fn test_run_limits(run_limits in run_limits()) {
        client().build_and_run(&BuildRunRequest {
            build: BuildRequest {
                environment: "rust".into(),
                files: vec![File {
                    name: "test.rs".into(),
                    content: formatdoc! {r#"
                        fn main() {{
                            println!("Hello World!");
                        }}
                    "#}
                }],
                compile_limits: Default::default(),
            },
            run: RunRequest{run_limits, ..Default::default()},
        }).unwrap();
    }
}

proptest! {
    #[test]
    #[ignore]
    fn test_run_args(stdin in stdin(256), args in args(100, 256), files in files(0, 10, 256)) {
        let expected = format!("{}\n{}\n{}", args.len() + 1, files.len(), stdin.as_ref().map(|s| s.chars().count()).unwrap_or(0));
        let result = client().build_and_run(&BuildRunRequest {
            build: BuildRequest {
                environment: "python".into(),
                files: vec![File {
                    name: "test.py".into(),
                    content: formatdoc! {r#"
                        import sys, os
                        print(len(sys.argv))
                        print(len(os.listdir()))
                        print(len(sys.stdin.read()))
                    "#}
                }],
                compile_limits: Default::default(),
            },
            run: RunRequest{files, stdin, args, run_limits: Default::default()},
        }).unwrap();
        assert_eq!(result.run.status, 0);
        assert_eq!(result.run.stdout.trim(), expected);
        assert!(result.run.stderr.is_empty());
    }
}

proptest! {
    #[test]
    #[ignore]
    fn random_bullshit_go(
        environment in valid_environment(),
        build_files in files(1, 4, 16),
        compile_limits in compile_limits(),
        stdin in stdin(32),
        args in args(8, 16),
        run_files in files(0, 4, 16),
        run_limits in run_limits()
    ) {
        client().build_and_run(&BuildRunRequest {
            build: BuildRequest {
                environment: environment.to_owned(),
                files: build_files,
                compile_limits
            },
            run: RunRequest {
                stdin,
                args,
                files: run_files,
                run_limits
            }
        }).ok();
    }
}

prop_compose! {
    fn filename() (name in "[a-zA-Z0-9._-]{1,32}".prop_filter("Invalid filename", |x| {
        let invalid_names = Lazy::new(|| Regex::new(r"^\.*$").unwrap());
        !invalid_names.is_match(x)
    })) -> String {
        name
    }
}

prop_compose! {
    fn src_file(max_len: usize) (name in filename(), content in string_regex(&format!("(?s).{{0,{max_len}}}")).unwrap()) -> File {
        File {name, content}
    }
}

prop_compose! {
    fn files(min_cnt: usize, max_cnt: usize, max_len: usize) (cnt in min_cnt..=max_cnt) (files in collection::vec(src_file(max_len), cnt)
        .prop_filter("Filenames must be unique",
            |files| files
                .iter()
                .map(|f| &f.name)
                .collect::<HashSet<_>>().len() == files.len())) -> Vec<File> {
        files
    }
}

prop_compose! {
    fn args(max_cnt: usize, max_len: usize) (cnt in 0..=max_cnt) (args in collection::vec(string_regex(&format!("[^\0]{{0,{max_len}}}")).unwrap(), cnt)) -> Vec<String> {
        args
    }
}

prop_compose! {
    fn stdin(max_len: usize) (stdin in option::of(string_regex(&format!("(?s).{{0,{max_len}}}")).unwrap())) -> Option<String> {
        stdin
    }
}

prop_compose! {
    fn valid_environment() (idx in 0usize..ENVIRONMENTS.len()) -> &'static str {
        ENVIRONMENTS[idx]
    }
}

include!(env!("LIMITS_TEST_SRC"));
include!(env!("ENVIRONMENTS_LIST_SRC"));
