use std::collections::HashMap;

use key_lock::KeyLock;
use poem_ext::response;
use poem_openapi::{param::Path, payload::Json, OpenApi};
use uuid::Uuid;

use crate::{
    config::Config,
    environments::Environments,
    program::{
        create_program, delete_program, run_program, CreateProgramError, DeleteProgramError,
        RunProgramError,
    },
    schemas::{
        CreateProgramRequest, CreateResult, CreateRunResult, Environment, RunProgramRequest,
        RunRequest, RunResult,
    },
};

pub struct Api {
    pub config: Config,
    pub environments: Environments,
    pub compile_lock: KeyLock<Uuid>,
}

#[OpenApi(tag = "Tags::Main")]
impl Api {
    /// Return a list of all environments.
    #[oai(path = "/environments", method = "get")]
    async fn list_environments(&self) -> ListEnvironments::Response {
        ListEnvironments::ok(
            self.environments
                .environments
                .iter()
                .map(|(id, env)| {
                    (
                        id.clone(),
                        Environment {
                            name: env.name.clone(),
                            version: env.version.clone(),
                        },
                    )
                })
                .collect(),
        )
    }

    /// Upload and immediately run a program.
    #[oai(path = "/run", method = "post")]
    async fn run(&self, data: Json<RunRequest>) -> Run::Response {
        let CreateResult {
            program_id,
            compile_result,
        } = match create_program(
            &self.config,
            &self.environments,
            data.0.create,
            &self.compile_lock,
        )
        .await
        {
            Ok(result) => result,
            Err(CreateProgramError::EnvironmentNotFound(_)) => return Run::environment_not_found(),
            Err(CreateProgramError::CompilationFailed(result)) => {
                return Run::compile_error(result)
            }
            Err(err) => return Err(err.into()),
        };

        match run_program(&self.config, &self.environments, program_id, data.0.run).await {
            Ok(run_result) => Run::ok(CreateRunResult {
                create: compile_result,
                run: run_result,
            }),
            Err(err) => Err(err.into()),
        }
    }

    /// Upload and compile a program.
    #[oai(path = "/programs", method = "post")]
    async fn create_program(&self, data: Json<CreateProgramRequest>) -> CreateProgram::Response {
        match create_program(&self.config, &self.environments, data.0, &self.compile_lock).await {
            Ok(result) => CreateProgram::ok(result),
            Err(CreateProgramError::EnvironmentNotFound(_)) => {
                CreateProgram::environment_not_found()
            }
            Err(CreateProgramError::CompilationFailed(result)) => {
                CreateProgram::compile_error(result)
            }
            Err(err) => Err(err.into()),
        }
    }

    /// Run a program that has been uploaded previously.
    #[oai(path = "/programs/:program_id/run", method = "post")]
    async fn run_program(
        &self,
        program_id: Path<Uuid>,
        data: Json<RunProgramRequest>,
    ) -> RunProgram::Response {
        match run_program(&self.config, &self.environments, program_id.0, data.0).await {
            Ok(result) => RunProgram::ok(result),
            Err(RunProgramError::ProgramNotFound) => RunProgram::not_found(),
            Err(err) => Err(err.into()),
        }
    }

    /// Delete a program.
    #[oai(path = "/programs/:program_id", method = "delete")]
    async fn delete_program(&self, program_id: Path<Uuid>) -> DeleteProgram::Response {
        match delete_program(&self.config, program_id.0).await {
            Ok(_) => DeleteProgram::ok(),
            Err(DeleteProgramError::ProgramNotFound) => DeleteProgram::not_found(),
            Err(err) => Err(err.into()),
        }
    }
}

response!(ListEnvironments = {
    Ok(200) => HashMap<String, Environment>,
});

response!(Run = {
    /// Code has been executed successfully.
    Ok(200) => CreateRunResult,
    /// Environment does not exist.
    EnvironmentNotFound(404, error),
    /// Code could not be compiled.
    CompileError(400, error) => RunResult,
});

response!(CreateProgram = {
    Ok(201) => CreateResult,
    /// Environment does not exist.
    EnvironmentNotFound(404, error),
    /// Code could not be compiled.
    CompileError(400, error) => RunResult,
});

response!(RunProgram = {
    Ok(200) => RunResult,
    /// Program does not exist.
    NotFound(404, error),
});

response!(DeleteProgram = {
    Ok(200),
    /// Program does not exist.
    NotFound(404, error),
});

#[derive(poem_openapi::Tags)]
enum Tags {
    Main,
}
