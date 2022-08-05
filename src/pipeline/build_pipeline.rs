/*
   Copyright 2021 JFrog Ltd

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

       http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.
*/

use actix_web::{get, put, web, HttpResponse, Responder, Scope};
use async_std::process;
use serde::Deserialize;
use std::fs;
use std::io;
use std::path::PathBuf;

use crate::pipeline::states::{BuildInfo, BuildStates, BuildStatus};

#[derive(Debug, Deserialize)]
struct GetArtifactRequest {
    build_id: String,
    filename: String,
}

#[derive(Clone, Deserialize, strum_macros::Display)]
pub enum PackageType {
    Docker,
    Maven2,
}

#[derive(Deserialize)]
pub enum SourceRepository {
    Git { url: String, tag: String },
}

#[derive(Deserialize)]
pub struct MappingInfo {
    pub package_type: PackageType,
    pub package_specific_id: String,
    pub source_repository: Option<SourceRepository>,
    pub build_spec_url: Option<String>,
}

#[get("{build_id}")]
async fn get_build(
    path: web::Path<String>,
    build_states: web::Data<BuildStates>,
) -> impl Responder {
    match build_states.get_build_info(&path.into_inner()).await {
        Some(build_info) => HttpResponse::Ok().json(build_info),
        None => HttpResponse::NotFound().finish(),
    }
}

#[get("{build_id}/artifacts/{filename}")]
async fn get_build_artifact(
    path: web::Path<GetArtifactRequest>,
    build_states: web::Data<BuildStates>,
) -> io::Result<impl Responder> {
    let get_artifact_request = path.into_inner();
    match build_states
        .get_build_info(&get_artifact_request.build_id)
        .await
    {
        Some(build_info) => match &build_info.status {
            BuildStatus::Running => Ok(HttpResponse::Accepted().finish()),
            BuildStatus::Success { .. } => {
                let artifact_path = PathBuf::from(format!(
                    "/tmp/pyrsia-build-pipeline/{}/artifacts/{}",
                    get_artifact_request.build_id,
                    get_artifact_request.filename
                ));
                let artifact_data = fs::read(&artifact_path)?;
                Ok(HttpResponse::Ok()
                    .append_header(("Content-Type", "application/octet-stream"))
                    .body(artifact_data))
            }
            BuildStatus::Failure(ref error) => Ok(HttpResponse::Gone().body(error.to_string())),
        },
        None => Ok(HttpResponse::NotFound().finish()),
    }
}

#[put("")]
async fn start_build(
    mapping_info: web::Json<MappingInfo>,
    build_states: web::Data<BuildStates>,
) -> Result<impl Responder, io::Error> {
    println!(
        "Requesting build of {} for {}",
        mapping_info.package_type, mapping_info.package_specific_id
    );

    let id = uuid::Uuid::new_v4().to_string();
    let build_info = BuildInfo {
        id: id.clone(),
        status: BuildStatus::Running,
    };

    build_states
        .update_build_info(&id, build_info.clone())
        .await;

    let working_dir = PathBuf::from(format!("/tmp/pyrsia-build-pipeline/{}", id));
    fs::create_dir_all(&working_dir)?;

    let pipeline_build_script_src = fs::File::open(format!(
        "pipelines/build-{}.sh",
        mapping_info.package_type
    ))?;
    let mut pipeline_build_script_dest = PathBuf::from(&working_dir);
    pipeline_build_script_dest.push(format!("build-{}.sh", mapping_info.package_type));
    let pipeline_build_script_dest_file = fs::File::create(pipeline_build_script_dest)?;
    io::copy(
        &mut io::BufReader::new(pipeline_build_script_src),
        &mut io::BufWriter::new(pipeline_build_script_dest_file),
    )?;

    let mut command = process::Command::new("sh");
    command.current_dir(&working_dir);

    command.arg(format!("build-{}.sh", mapping_info.package_type))
        .arg(mapping_info.package_type.to_string())
        .arg(&mapping_info.package_specific_id)
        .arg(&id);

    match mapping_info.package_type {
        PackageType::Docker => {
            let items: Vec<&str> = if mapping_info.package_specific_id.contains('@') {
                mapping_info.package_specific_id.split('@').collect()
            } else {
                mapping_info.package_specific_id.split(':').collect()
            };
            if items[0].contains('/') {
                command.arg(items[0]);
            } else {
                command.arg(format!("library/{}", items[0]));
            }
            command.arg(items[1]);
        },
        PackageType::Maven2 => {
            let source_repository = match mapping_info.source_repository.as_ref().unwrap() {
                SourceRepository::Git { url, tag } => (url, tag),
            };
            command
                .arg(source_repository.0)
                .arg(source_repository.1);

            if let Some(build_spec_url) = mapping_info.build_spec_url.as_ref() {
                command.arg(build_spec_url);
            }
        }
    }

    tokio::spawn(async move {
        if let Err(io_error) = run_command(command, &id, build_states.clone()).await {
            build_states
                .update_build_info(
                    &id,
                    BuildInfo {
                        id: id.to_string(),
                        status: BuildStatus::Failure(io_error.to_string()),
                    },
                )
                .await;
        }
    });

    Ok(HttpResponse::Ok().json(build_info))
}

pub fn build_pipeline_service() -> Scope {
    web::scope("build")
        .service(get_build)
        .service(get_build_artifact)
        .service(start_build)
}

async fn run_command(
    mut command: process::Command,
    build_id: &str,
    build_states: web::Data<BuildStates>,
) -> io::Result<()> {
    println!("Starting build with ID {}", build_id);

    let exit_status = command.status().await?;

    let build_info = if exit_status.success() {
        let build_dir = PathBuf::from(format!("/tmp/pyrsia-build-pipeline/{}/artifacts", build_id));

        let mut artifact_urls = vec![];
        for entry in fs::read_dir(build_dir)? {
            let file = entry?;
            let file_type = file.file_type()?;
            if file_type.is_file() {
                if let Some(file_name) = file.file_name().to_str() {
                    artifact_urls.push(format!("/build/{}/artifacts/{}", build_id, file_name));
                }
            }
        }

        BuildInfo {
            id: build_id.to_string(),
            status: BuildStatus::Success {
                artifact_urls,
            },
        }
    } else {
        BuildInfo {
            id: build_id.to_string(),
            status: match exit_status.code() {
                Some(code) => BuildStatus::Failure(code.to_string()),
                None => BuildStatus::Failure("process exited with unknown reason".to_string()),
            },
        }
    };

    build_states.update_build_info(build_id, build_info).await;

    Ok(())
}
