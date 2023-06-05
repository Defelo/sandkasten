[![check](https://github.com/Defelo/sandkasten/actions/workflows/check.yml/badge.svg)](https://github.com/Defelo/sandkasten/actions/workflows/check.yml)
[![test](https://github.com/Defelo/sandkasten/actions/workflows/test.yml/badge.svg)](https://github.com/Defelo/sandkasten/actions/workflows/test.yml)
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
- [ ] Add more packages.

## API Documentation
On a running Sandkasten instance, the API documentation is available on `<instance>/docs` and
`<instance>/redoc`. There is also an OpenAPI specification available on `<instance>/openapi.json`.

## Public Instance
Not available (yet).

## Setup instructions
The recommended way of installing Sandkasten is to setup a dedicated virtual machine running NixOS.
To make this setup easier, this repository contains a basic NixOS configuration and an installation
script.

### NixOS VM Setup
The following steps have been tested on Proxmox VE 7.4-3 x86_64.

1. Download the minimal NixOS ISO image from https://nixos.org/download.html#nixos-iso
2. Create a new virtual machine.
    - Disk size: at least 16GB
    - Internet connection and DHCP server to get an IPv4 address are required.
3. Start the virtual machine and boot into the NixOS installer.
4. Run `sudo su` to obtain root privileges.
5. If necessary, change the keyboard layout (e.g. `loadkeys de` for german qwertz layout).
6. Use `lsblk` or `fdisk -l` to find the name of your hard disk.
7. Run the following commands to download the installation script from GitHub and start the
    installation. Replace `[disk]` with the path to your hard disk (e.g. `/dev/sda`). Note that
    this will erase all data on the disk you specify.
    ```bash
    curl -o install-vm.sh https://raw.githubusercontent.com/Defelo/sandkasten/develop/install-vm.sh
    bash install-vm.sh [disk]
    ```
8. After the script is done, the vm will reboot into the new NixOS installation. The initial root
    password is `sandkasten` if you want to login via ssh. The Sandkasten server is started
    automatically and should be listening on `0.0.0.0:80` by default.

In `/root/sandkasten` you can find a git checkout of this repository. To update Sandkasten run
`git pull && nixos-rebuild switch --flake .` in this directory. In `nix/nixos/vm.nix` you can also
find the NixOS configuration of the vm. After making changes to this file run
`nixos-rebuild switch --flake .` to apply them.

### NixOS Module
Follow these steps if you want to install Sandkasten on an existing (flakes based) NixOS
installation:

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
        environments = p: with p; [
          rust python typescript  # use `all` to install all environments
        ];
        # example config:
        host = "0.0.0.0";
        port = 8080;
        max_concurrent_jobs = 16;
        run_limits.time = 10;
        # for a full list of configuration options, see `config.toml`
      };
    }
    ```

## Development

### Setup instructions

#### Required software
The following components are needed for a working development environment:

- [Rust](https://www.rust-lang.org/) (stable) toolchain
- [Nix](https://nixos.org/) with [flakes](https://nixos.wiki/wiki/Flakes) enabled
- [direnv](https://github.com/direnv/direnv) (optional, but recommended)

#### Enter the development shell
If you have direnv installed, you can just use
`direnv allow` to setup your shell for development. Otherwise you can also use `nix develop`
to enter a development shell. This will add some tools to your `PATH` and set a few environment
variables that are needed by Sandkasten and some of the integration tests.

#### Setup nsjail
The first time you enter
the development shell, you should run the `setup-nsjail` command, which will copy the `nsjail`
binary into your current working directoy, `chown` it to `root` and set the `setuid` bit to allow
Sandkasten to run this binary as root without having to run Sandkasten itself as root (but of
course you could also do that).

#### Install Sandkasten packages
Before starting Sandkasten, you should setup a Nix profile with the environments that you want to be
available on your instance. A full list of installable environments is available at
[nix/packages](https://github.com/Defelo/sandkasten/tree/develop/nix/packages). To install a
package, you can use the following command:

```bash
nix profile install --profile pkgs .#packages.<package-name>
```

If you want to install all packages, use `all` for `<package-name>`. You can also add or remove
packages later, but you need to restart Sandkasten after doing so.

#### Start the application
In the development shell you can just use `cargo run` to start Sandkasten.

### Unit tests
To run the unit tests, you can just use `cargo test`. This only requires you to have a working rust
toolchain, but you should not need to setup nix for this.

### Integration tests
To run the integration tests, you can use `cargo test -F nix -- --ignored`. For this to work you
need to have a Sandkasten instance running on `127.0.0.1:8000`. You can also specify a different
instance via the `TARGET_HOST` environment variable. If you only want to run the integration tests
that do not require a nix development shell, you can omit the `-F nix`. In the development shell you
can also run the `integration-tests` command to automatically start a temporary sandkasten instance
and run the integration tests against it. There is also a `cov` command that runs the integration
tests and writes an html coverage report to `lcov_html/index.html`.

### Packages
All packages are defined using nix expressions in
[nix/packages](https://github.com/Defelo/sandkasten/tree/develop/nix/packages). Each package has a
unique id, a human-readable name, a version, optionally a script to compile a program, a script to
run a program and a test program that is executed as part of the integration tests to ensure that
the package is working.

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
