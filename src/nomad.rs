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
    fn get(&self, resource: &str) -> Result<ureq::Response, ureq::Response> {
        let url = format!("{}/v1/{}", self.address, resource);
        let resp = ureq::get(&url).call();
        trace!("Response <{}> [{}]", url, resp.status());
        if resp.error() {
            return Err(resp);
        }
        Ok(resp)
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
    pub fn get_jobs(&self) -> Result<Vec<JobListing>, String> {
        let jobs: Vec<JobListing> = match self.client.get("jobs") {
            Ok(resp) => match resp.into_json() {
                Ok(buf) => serde_json::from_value(buf).expect("failed to decode response"),
                Err(_) => return Err(String::from("failed to read response")),
            },
            Err(resp) => return Err(resp.into_string().unwrap()),
        };
        return Ok(jobs);
    }

    pub fn get_job(&self, id: &str) -> Result<Job, &'static str> {
        let job: Job = match self.client.get(&format!("job/{}", id)).unwrap().into_json() {
            Ok(buf) => serde_json::from_value(buf).expect("failed to decode response"),
            Err(_) => return Err("failed to read response"),
        };
        Ok(job)
    }
}
