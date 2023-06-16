[![GitHub](https://img.shields.io/static/v1?label=GitHub&message=Defelo/sandkasten&style=for-the-badge&logo=github)](https://github.com/Defelo/sandkasten)
[![GitHub](https://img.shields.io/github/stars/Defelo/sandkasten?style=for-the-badge&logo=github)](https://github.com/Defelo/sandkasten)
[![GitHub](https://img.shields.io/github/forks/Defelo/sandkasten?style=for-the-badge&logo=github)](https://github.com/Defelo/sandkasten)
[![GitHub](https://img.shields.io/github/license/Defelo/sandkasten?style=for-the-badge)](https://github.com/Defelo/sandkasten)
[![GitHub](https://img.shields.io/static/v1?label=OpenAPI%20Spec&message=/openapi.json&style=for-the-badge&logo=openapiinitiative)](openapi.json)

## What is this?
Sandkasten is a code execution engine for running arbitrary untrusted/harmful code in a sandbox,
isolating it from both the host system and other Sandkasten jobs. A simple REST API allows uploading
and executing arbitrary programs, while also enabling the user to specify resource limits and
providing feedback on the actual resources used. This project was partly inspired by
[Piston](https://github.com/engineer-man/piston) and aims to solve some problems with it.

## Features
- Compile and execute arbitrary programs.
- Cache compilation results to avoid having to recompile the same programs for every time they
  are run.
- Set resource limits for both compile and run steps.
- Report resource usage for both compile and run steps.
- Packages are defined using [Nix](https://nixos.org/).
- Programs are deleted automatically if they are not executed anymore.
- Specify stdin, command line arguments and files in the working directory for run steps.
- Specify environment variables for both compile and run steps.
- Client library for Rust ([crate](https://crates.io/crates/sandkasten-client), [documentation](https://docs.rs/sandkasten-client))

## API Documentation
The API documentation is available on [`/docs`](docs) and [`/redoc`](/redoc). There is also an
OpenAPI specification available on [`/openapi.json`](openapi.json).
