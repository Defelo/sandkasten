[![check](https://github.com/Defelo/sandkasten/actions/workflows/check.yml/badge.svg)](https://github.com/Defelo/sandkasten/actions/workflows/check.yml)
[![test](https://github.com/Defelo/sandkasten/actions/workflows/test.yml/badge.svg)](https://github.com/Defelo/sandkasten/actions/workflows/test.yml)
[![docker](https://github.com/Defelo/sandkasten/actions/workflows/docker.yml/badge.svg)](https://github.com/Defelo/sandkasten/actions/workflows/docker.yml)
[![codecov](https://codecov.io/gh/Defelo/sandkasten/branch/develop/graph/badge.svg?token=Y5CE2887KO)](https://codecov.io/gh/Defelo/sandkasten)
![Version](https://img.shields.io/github/v/tag/Defelo/sandkasten?include_prereleases&label=version)
[![dependency status](https://deps.rs/repo/github/Defelo/sandkasten/status.svg)](https://deps.rs/repo/github/Defelo/sandkasten)

# Sandkasten
Run untrusted code in an isolated environment

## What is this?
Sandkasten is a code execution engine for running arbitrary untrusted/harmful code in a sandbox,
isolating it from both the host system and other Sandkasten jobs. A simple REST API allows uploading
and executing arbitrary programs, while also enabling the user to specify resource limits and
providing feedback on the actual resources used. This project was partly inspired by
[Piston](https://github.com/engineer-man/piston) and aims to solve some problems with it.

## How does it work?
Sandkasten uses [nsjail](https://github.com/google/nsjail) to run programs in restricted
environments and to enforce the specified resource limits. Additionally
[GNU Time](https://www.gnu.org/software/time/) is used for reporting the resources used by the
program. Programs are always run in a chroot environment using nsjail, which contains only the
following directories:

- `/program` (rw in compile steps, ro in run steps) contains the compiled program
- `/box` (ro) current working directory which contains the specified files for compile/run steps
- `/tmp` (rw, tmpfs)
- the paths in `/nix/store` that are needed by the selected environment (ro mount from host)
- some files in `/dev` and `/etc` which are needed for some packages to work properly

Programs are uniquely identified using the hash value of their source files and selected
environments. If a program has been uploaded and compiled before and is then uploaded again, the
same program id is used and the existing compilation results can be used without having to recompile
the program.

## Features
- [x] Compile and execute arbitrary programs.
- [x] Cache compilation results to avoid having to recompile the same programs for every time they
      are run.
- [x] Set resource limits for both compile and run steps.
- [x] Report resource usage for both compile and run steps.
- [x] Packages are defined via Nix.
- [x] Programs are deleted automatically if they are not executed anymore.
- [x] Specify stdin, command line arguments and files in the working directory for run steps.
- [x] Specify environment variables for both compile and run steps.
- [x] Client library for rust ([crate](https://crates.io/crates/sandkasten-client), [documentation](https://docs.rs/sandkasten-client))

### Planned/Ideas
- [ ] Communicate with running programs via websockets.
- [ ] Spawn multiple processes that can communicate with each other.
- [ ] JWTs for individual limits (+rate limits).
- [ ] Install packages at runtime via nix.
- [ ] Add more packages.

## API Documentation
On a running Sandkasten instance, the API documentation is available on `<instance>/docs` and
`<instance>/redoc`. There is also an OpenAPI specification available on `<instance>/openapi.json`.

## Public Instance
Not available (yet).

## Setup instructions

### NixOS Module

1. Add this repository to your flake inputs:
    ```nix
    {
      inputs.sandkasten.url = "github:Defelo/sandkasten";
    }
    ```
2. Add the module to your NixOS configuration:
    ```nix
    {
      imports = [sandkasten.nixosModules.sandkasten];
    }
    ```
3. Configure the module:
    ```nix
    {
      services.sandkasten = {
        enable = true;
        redis_url = "redis+unix:///${config.services.redis.servers.sandkasten.unixSocket}";
        # example config:
        host = "0.0.0.0";
        port = 8080;
        max_concurrent_jobs = 16;
        run_limits.time = 10;
        # for a full list of configuration options, see `config.toml`
      };
      services.redis.servers.sandkasten = {
        enable = true;
        user = "sandkasten";
      };
    }
    ```

### Nix flake
```bash
nix shell nixpkgs#redis --command redis-server &
CONFIG_PATH=config.toml nix run github:Defelo/sandkasten
```

### Docker
```bash
docker compose up -d
```

**Warning:** Unfortunately the `cgroup` options of `nsjail` do not work when running in Docker.
Therefore only the `rlimit` options are set to enforce the resource limits, which means that for
example the memory limit applies to each process individually (e.g. if your resource limits are
`memory=1024` and `processes=64`, a single program could consume a total of 64GB of memory by
spawning 64 processes that all consume 1GB individually).

## Development

### Setup instructions
The following components are needed for a working development environment:

- [Rust](https://www.rust-lang.org/) (stable) toolchain
- [Nix](https://nixos.org/) with [flakes](https://nixos.wiki/wiki/Flakes) enabled

If you also have [direnv](https://github.com/direnv/direnv) installed, you can just use
`direnv allow` to setup your shell for development. Otherwise you can also use `nix develop`
to enter a development shell. This will add some tools to your `PATH` and set a few environment
variables that are needed by Sandkasten and some of the integration tests. In the development shell
you can just use `cargo run` to start the application.

### Unit tests
To run the unit tests, you can just use `cargo test`. This only requires you to have a working rust
toolchain, but you should not need to setup nix for this.

### Integration tests
To run the integration tests, you can use `cargo test -F nix -- --ignored`. For this to work you
need to have a Sandkasten instance running on `127.0.0.1:8000`. You can also specify a different
instance via the `TARGET` environment variable. If you only want to run the integration tests that
do not require a nix development shell, you can omit the `-F nix`. In the development shell you can
also run the `integration-tests` command to automatically start a temporary sandkasten instance and
run the integration tests against it. There is also a `cov` command that runs the integration tests
and writes an html coverage report to `lcov_html/index.html`.

### Packages
All packages are defined using nix expressions in
[nix/packages](https://github.com/Defelo/sandkasten/tree/develop/nix/packages). Each package has a
unique id, a human-readable name, a version, optionally a script to compile a program, a script to
run a program and a test program that is executed as part of the integration tests to ensure that
the package is working. When creating a new package, don't forget to add it to
[nix/packages/default.nix](https://github.com/Defelo/sandkasten/blob/develop/nix/packages/default.nix).

#### Compile scripts
The compile script of a package is executed whenever a new program has been uploaded. When this
script is run, the current working directory (`/box`) contains all the source files and the command
line arguments contain the names of the source files in the same order as they were specified by
the client (starting with `main_file` which represents the entrypoint into the program). The purpose
of the compile script is to compile the provided program and store the result (plus any files that
may be needed to run the program) in `/program`.

If a package does not have a compile script, the source files are instead copied directly into the
program directory.

#### Run scripts
The run script of a package is executed whenever a program is executed. When this script is run, the
current working directory (`/box`) contains the files that have been specified in the run step (if
any) and `/program` contains the files that have been produced by the corresponding compile script
previously (or the source files if the packages does not have a compile script). The first command
line argument is always the name of the `main_file` (which represents the entrypoint into the
program). In most cases, this is only relevant for interpreted languages (like Python) and can be
ignored for most compiled languages. All other command line arguments are the ones specified by the
client and should be forwarded to the actual program.

#### Test program
Every package should provide a test program that checks the following:

- Multiple source files are working (e.g. `first_file.py` can import `second_file.py`)
- Reading from stdin is working. The program should assert that the string `stdin` is read from
  stdin.
- Command line arguments are working. The program should assert that the only three command line
  arguments are `foo`, `bar` and `baz`.
- File system is working. The program should assert that the file `test.txt` in the current working
  directory contains the string `hello world`.

If any of these checks fails, the program should exit with a non-zero exit code. Otherwise, if all
checks passed, it should exit with exit code zero and print `OK` to stdout.
