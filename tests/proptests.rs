#![cfg(feature = "nix")]

use std::{
    collections::{BTreeMap, HashSet},
    fmt::Write,
};

use common::client;
use indoc::formatdoc;
use proptest::{collection, option, prelude::*, string::string_regex};
use sandkasten_client::schemas::programs::{
    BuildRequest, BuildRunRequest, EnvVar, File, LimitsOpt, MainFile, RunRequest,
};

mod common;

proptest! {
    #[test]
    #[ignore]
    fn test_files(main_file in main_file(256), build_files in files(1, 10, 256), run_files in files(0, 10, 256)) {
        prop_assume!(build_files.iter().all(|f| f.name != main_file.name.as_deref().unwrap_or("code.py")));
        client().build_and_run(&BuildRunRequest {
            build: BuildRequest {
                environment: "python".into(),
                main_file,
                files: build_files,
                env_vars: vec![],
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
                main_file: MainFile {
                    name: Some("test.rs".into()),
                    content: formatdoc! {r#"
                        fn main() {{
                            // {compile_limits:?}
                            println!("Hello World!");
                        }}
                    "#}
                },
                compile_limits,
                ..Default::default()
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
                main_file: MainFile {
                    name: Some("test.rs".into()),
                    content: formatdoc! {r#"
                        fn main() {{
                            println!("Hello World!");
                        }}
                    "#}
                },
                ..Default::default()
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
                main_file: MainFile {
                    name: Some("test.py".into()),
                    content: formatdoc! {r#"
                        import sys, os
                        print(len(sys.argv))
                        print(len(os.listdir()))
                        print(len(sys.stdin.read()))
                    "#}
                },
                ..Default::default()
            },
            run: RunRequest{files, stdin, args, env_vars: vec![], run_limits: Default::default()},
        }).unwrap();
        assert_eq!(result.run.status, 0);
        assert_eq!(result.run.stdout.trim(), expected);
        assert!(result.run.stderr.is_empty());
    }
}

proptest! {
    #[test]
    #[ignore]
    fn test_env_vars(build_vars in env_vars_map(16, 256), run_vars in env_vars_map(16, 256)) {
        let expected = build_vars.iter().chain(&run_vars).map(|(_, v)| v.as_str()).collect::<String>();
        let mut src = String::new();
        write!(&mut src, "fn main() {{").unwrap();
        for name in build_vars.keys() {
            write!(&mut src, "print!(\"{{}}\", env!(\"{name}\"));").unwrap();
        }
        for name in run_vars.keys() {
            write!(&mut src, "print!(\"{{}}\", std::env::var(\"{name}\").unwrap());").unwrap();
        }
        write!(&mut src, "}}").unwrap();
        let result = client().build_and_run(&BuildRunRequest {
            build: BuildRequest {
                environment: "rust".into(),
                main_file: MainFile {
                    name: Some("test.rs".into()),
                    content: src,
                },
                env_vars: build_vars.into_iter().map(|(name, value)| EnvVar {name, value}).collect(),
                ..Default::default()
            },
            run: RunRequest {env_vars: run_vars.into_iter().map(|(name, value)| EnvVar {name, value}).collect(), ..Default::default()},
        }).unwrap();
        assert_eq!(result.run.status, 0);
        assert_eq!(result.run.stdout, expected);
        assert!(result.run.stderr.is_empty());
    }
}

proptest! {
    #[test]
    #[ignore]
    fn random_bullshit_go(
        environment in valid_environment(),
        main_file in main_file(16),
        build_files in files(1, 4, 16),
        build_env_vars in env_vars(8, 16),
        compile_limits in compile_limits(),
        stdin in stdin(32),
        args in args(8, 16),
        run_files in files(0, 4, 16),
        run_env_vars in env_vars(8, 16),
        run_limits in run_limits()
    ) {
        prop_assume!(build_files
            .iter()
            .all(|f| if let Some(name) = main_file.name.as_ref() {
                &f.name != name
            } else {
                !f.name.starts_with("code.")
            }));
        client().build_and_run(&BuildRunRequest {
            build: BuildRequest {
                environment: environment.to_owned(),
                main_file,
                files: build_files,
                env_vars: build_env_vars,
                compile_limits
            },
            run: RunRequest {
                stdin,
                args,
                files: run_files,
                env_vars: run_env_vars,
                run_limits
            }
        }).ok();
    }
}

prop_compose! {
    fn filename() (name in "[a-zA-Z0-9._-]{1,32}".prop_filter("Invalid filename", |x| {
        !x.chars().all(|c| c == '.')
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
    fn main_file(max_len: usize) (name in option::of(filename()), content in string_regex(&format!("(?s).{{0,{max_len}}}")).unwrap()) -> MainFile {
        MainFile {name, content}
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
    fn env_var(max_len: usize) (name in "[a-zA-Z0-9_]{1,64}".prop_filter("Invalid name", |x| {
        x != "_"
    }), value in string_regex(&format!("[^\0]{{0,{max_len}}}")).unwrap()) -> EnvVar {
        EnvVar {name, value}
    }
}

prop_compose! {
    fn env_vars(max_cnt: usize, max_len: usize) (cnt in 0..=max_cnt) (env_vars in collection::vec(env_var(max_len), cnt)) -> Vec<EnvVar> {
        env_vars
    }
}

prop_compose! {
    fn env_vars_map(max_cnt: usize, max_len: usize) (env_vars in env_vars(max_cnt, max_len)) -> BTreeMap<String, String> {
        env_vars.into_iter().map(|v| (v.name, v.value)).collect()
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
