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

use serde::Serialize;
use std::collections::HashMap;
use tokio::sync::RwLock;

#[derive(Clone, Debug, Serialize)]
pub struct BuildArtifact {
    pub artifact_url: String,
    pub source_artifact_url: String,
}

#[derive(Clone, Debug, Serialize)]
pub enum BuildStatus {
    Running,
    Success { artifacts: Vec<BuildArtifact> },
    Failure(String),
}

#[derive(Clone, Debug, Serialize)]
pub struct BuildInfo {
    pub id: String,
    pub status: BuildStatus,
}

pub struct BuildStates {
    states: RwLock<HashMap<String, BuildInfo>>,
}

impl BuildStates {
    pub fn new() -> Self {
        BuildStates {
            states: RwLock::new(Default::default()),
        }
    }

    pub async fn update_build_info(&self, key: &str, build_info: BuildInfo) {
        let mut states_write = self.states.write().await;
        states_write.insert(key.to_string(), build_info);
    }

    pub async fn get_build_info(&self, key: &str) -> Option<BuildInfo> {
        let states_read = self.states.read().await;
        states_read.get(key).cloned()
    }
}

impl Default for BuildStates {
    fn default() -> Self {
        BuildStates::new()
    }
}
