use anyhow::{anyhow, Result};
use log::trace;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Clone, Debug, Default)]
pub struct Client {
    address: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct JobListing {
    pub ID: String,
    pub ParentID: String,
    pub Name: String,
    pub Type: String,
    pub Status: String,
    pub ParameterizedJob: Option<bool>,
    pub Periodic: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Job {
    #[serde(flatten)]
    listing: JobListing,

    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

impl Client {
    /// Issue an HTTP Get against the given resource.
    ///
    /// # Arguments
    ///
    /// * `resource` the path to the resource being fetched.
    fn get(&self, resource: &str) -> Result<ureq::Response> {
        let url = format!("{}/v1/{}", self.address, resource);
        let resp = ureq::get(&url).call();
        trace!("Response <{}> [{}]", url, resp.status());
        match resp.synthetic_error() {
            Some(resp) => {
                let msg = if resp.to_string().contains("Connection refused") {
                    format!("Could not connect to server at {}", &self.address)
                } else {
                    format!("{}: {}", resp.status(), resp.to_string())
                };
                Err(anyhow!(msg))
            }
            None => Ok(resp),
        }
    }
}

pub fn get_client() -> Client {
    Client {
        address: match std::env::var("NOMAD_ADDR") {
            Ok(addr) => addr,
            Err(_) => String::from("http://127.0.0.1:4646"),
        },
    }
}

pub struct Nomad {
    pub client: Client,
}

impl Nomad {
    /// Get all jobs in the cluster.
    pub fn get_jobs(&self) -> Result<Vec<JobListing>> {
        let jobs: Vec<JobListing> = match self.client.get("jobs") {
            Ok(resp) => match resp.into_json() {
                Ok(buf) => serde_json::from_value(buf).expect("failed to decode response"),
                Err(_) => return Err(anyhow!("failed to read response")),
            },
            Err(err) => return Err(err),
        };
        return Ok(jobs);
    }

    /// Get a job by its ID.
    ///
    /// # Arguments
    ///
    /// * `id` - the ID of the job to retrieve.
    pub fn get_job(&self, id: &str) -> Result<Job> {
        let job: Job = match self.client.get(&format!("job/{}", id)).unwrap().into_json() {
            Ok(buf) => serde_json::from_value(buf).expect("failed to decode response"),
            Err(_) => return Err(anyhow!("failed to read response")),
        };
        Ok(job)
    }
}
