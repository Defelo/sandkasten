use std::collections::HashMap;

use poem_ext::response;
use poem_openapi::{param::Path, payload::Json, OpenApi};
use uuid::Uuid;

use crate::{
    config::Config,
    environments::Environments,
    program::{create_program, delete_program, run_program, CreateProgramError},
    schemas::{
        CreateProgramRequest, CreateResult, Environment, RunProgramRequest, RunRequest, RunResult,
    },
};

pub struct Api {
    pub config: Config,
    pub environments: Environments,
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
        // match create_program(&self.config, &self.environments, data.0.create).await {
        //     Ok(result) => Run::ok(
        //         run_program(
        //             &self.config,
        //             &self.environments,
        //             result.program_id,
        //             data.0.run,
        //         )
        //         .await?,
        //     ),
        //     Err(CreateProgramError::EnvironmentNotFound(_) => Run::)
        // }
        todo!()
    }

    /// Upload and compile a program.
    #[oai(path = "/programs", method = "post")]
    async fn create_program(&self, data: Json<CreateProgramRequest>) -> CreateProgram::Response {
        match create_program(&self.config, &self.environments, data.0).await {
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
        RunProgram::ok(run_program(&self.config, &self.environments, program_id.0, data.0).await?)
    }

    /// Delete a program.
    #[oai(path = "/programs/:program_id", method = "delete")]
    async fn delete_program(&self, program_id: Path<Uuid>) -> DeleteProgram::Response {
        delete_program(&self.config, &self.environments, program_id.0).await?;
        DeleteProgram::ok()
    }
}

response!(ListEnvironments = {
    Ok(200) => HashMap<String, Environment>,
});

response!(Run = {
    /// Code has been executed successfully.
    Ok(200) => RunResult,
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
