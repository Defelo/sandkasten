use std::collections::HashSet;

use indoc::formatdoc;
use once_cell::unsync::Lazy;
use proptest::{collection, option, prelude::*};
use regex::Regex;
use sandkasten::schemas::programs::{BuildRequest, BuildRunRequest, File, LimitsOpt, RunRequest};

use common::build_and_run;

mod common;

proptest! {
    #[test]
    #[ignore]
    fn test_files(build_files in files(1), run_files in files(0)) {
        build_and_run(&BuildRunRequest {
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
        build_and_run(&BuildRunRequest {
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
        build_and_run(&BuildRunRequest {
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
    fn test_run_args(stdin in option::of("(?s).{0,2048}"), args in args(), files in files(0)) {
        let expected = format!("{}\n{}\n{}", args.len() + 1, files.len(), stdin.as_ref().map(|s| s.chars().count()).unwrap_or(0));
        let result = build_and_run(&BuildRunRequest {
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

prop_compose! {
    fn filename() (name in "[a-zA-Z0-9._-]{1,32}".prop_filter("Invalid filename", |x| {
        let invalid_names = Lazy::new(|| Regex::new(r"^\.*$").unwrap());
        !invalid_names.is_match(x)
    })) -> String {
        name
    }
}

prop_compose! {
    fn src_file() (name in filename(), content in "(?s).{0,2048}") -> File {
        File {name, content}
    }
}

prop_compose! {
    fn files(min: usize) (cnt in min..10) (files in collection::vec(src_file(), cnt)
        .prop_filter("Filenames must be unique",
            |files| files
                .iter()
                .map(|f| &f.name)
                .collect::<HashSet<_>>().len() == files.len())) -> Vec<File> {
        files
    }
}

prop_compose! {
    fn args() (cnt in 0usize..100) (args in collection::vec("[^\0]{0,256}", cnt)) -> Vec<String> {
        args
    }
}

include!(env!("LIMITS_TEST_SRC"));
