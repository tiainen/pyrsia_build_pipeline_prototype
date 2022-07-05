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

use clap::Parser;

const DEFAULT_HOST: &str = "127.0.0.1";
const DEFAULT_PORT: &str = "8080";

/// Application to start pipelines for building artifacts for the Pyrsia network
#[derive(Debug, Parser)]
#[clap(name = "Pyrsia Build Pipeline")]
pub struct PyrsiaBuildPipelineArgs {
    /// The host address to bind to for the http server
    #[clap(long, short = 'H', default_value = DEFAULT_HOST)]
    pub host: String,
    /// the port to listen to for the http server
    #[clap(long, short, default_value = DEFAULT_PORT)]
    pub port: u16,
}
