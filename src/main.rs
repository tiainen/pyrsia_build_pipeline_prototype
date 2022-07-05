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

pub mod args;
pub mod pipeline;

use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use clap::Parser;

use args::parser::PyrsiaBuildPipelineArgs;
use pipeline::build_pipeline::build_pipeline_service;
use pipeline::states::BuildStates;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    pretty_env_logger::init();

    let args = PyrsiaBuildPipelineArgs::parse();

    let app_data = web::Data::new(BuildStates::new());

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(app_data.clone())
            .service(build_pipeline_service())
    })
    .bind((args.host, args.port))?
    .run()
    .await
}
