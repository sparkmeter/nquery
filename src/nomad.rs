use anyhow::{anyhow, Result};
use log::trace;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Client {
    address: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct ParameterizedJob {
    pub Payload: String,
    pub MetaRequired: Option<Vec<String>>,
    pub MetaOptional: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct Periodic {
    pub Enabled: bool,
    pub Spec: String,
    pub SpecType: String,
    pub ProhibitOverlap: bool,
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
#[allow(non_snake_case)]
pub struct Job {
    #[serde(flatten)]
    listing: JobListing,
    // Annoyingly, these fields have different types in a fullly-defined Job object
    pub ParameterizedJob: Option<ParameterizedJob>,
    pub Periodic: Option<Periodic>,

    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

pub trait NomadClient {
    fn get(&mut self, resource: &str) -> Result<ureq::Response>;
}

impl NomadClient for Client {
    /// Issue an HTTP Get against the given resource.
    ///
    /// # Arguments
    ///
    /// * `resource` the path to the resource being fetched.
    fn get(&mut self, resource: &str) -> Result<ureq::Response> {
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

/// Get the Nomad client
pub fn get_client() -> Client {
    Client {
        address: match std::env::var("NOMAD_ADDR") {
            Ok(addr) => addr,
            Err(_) => String::from("http://127.0.0.1:4646"),
        },
    }
}

/// Get all jobs in the cluster.
///
/// # Arguments
///
/// * `prefix` a string prefix which all the returned jobs must match
pub fn get_jobs(client: &mut dyn NomadClient, prefix: &str) -> Result<Vec<JobListing>> {
    let path = format!(
        "{}?prefix={}",
        "jobs",
        utf8_percent_encode(prefix, NON_ALPHANUMERIC).to_string()
    );
    let jobs: Vec<JobListing> = match client.get(&path) {
        Ok(resp) => match resp.into_json() {
            Ok(buf) => serde_json::from_value(buf).expect("failed to decode response"),
            Err(_) => return Err(anyhow!("failed to read response")),
        },
        Err(err) => return Err(err),
    };
    Ok(jobs)
}

/// Get a job by its ID.
///
/// # Arguments
///
/// * `id` - the ID of the job to retrieve.
pub fn get_job(client: &mut dyn NomadClient, id: &str) -> Result<Job> {
    let job: Job = match client.get(&format!("job/{}", id)).unwrap().into_json() {
        Ok(buf) => serde_json::from_value(buf)?,
        Err(_) => return Err(anyhow!("failed to read response")),
    };
    Ok(job)
}

#[cfg(test)]
mod test {
    use super::*;

    const FULL_JOB: &str = r#"{"Stop":false,"Region":"global","Namespace":"default","ID":"example","ParentID":"","Name":"example","Type":"service","Priority":50,"AllAtOnce":false,"Datacenters":["dc1"],"Constraints":null,"Affinities":null,"Spreads":null,"TaskGroups":[{"Name":"cache","Count":1,"Update":{"Stagger":30000000000,"MaxParallel":1,"HealthCheck":"checks","MinHealthyTime":10000000000,"HealthyDeadline":180000000000,"ProgressDeadline":600000000000,"AutoRevert":false,"AutoPromote":false,"Canary":0},"Migrate":{"MaxParallel":1,"HealthCheck":"checks","MinHealthyTime":10000000000,"HealthyDeadline":300000000000},"Constraints":null,"Scaling":null,"RestartPolicy":{"Attempts":2,"Interval":1800000000000,"Delay":15000000000,"Mode":"fail"},"Tasks":[{"Name":"redis","Driver":"docker","User":"","Config":{"image":"redis:3.2","port_map":[{"db":6379.0}]},"Env":null,"Services":[{"Name":"redis-cache","TaskName":"","PortLabel":"db","AddressMode":"auto","EnableTagOverride":false,"Tags":["global","cache"],"CanaryTags":null,"Checks":[{"Name":"alive","Type":"tcp","Command":"","Args":null,"Path":"","Protocol":"","PortLabel":"","Expose":false,"AddressMode":"","Interval":10000000000,"Timeout":2000000000,"InitialStatus":"","TLSSkipVerify":false,"Method":"","Header":null,"CheckRestart":null,"GRPCService":"","GRPCUseTLS":false,"TaskName":"","SuccessBeforePassing":0,"FailuresBeforeCritical":0}],"Connect":null,"Meta":null,"CanaryMeta":null}],"Vault":null,"Templates":null,"Constraints":null,"Affinities":null,"Resources":{"CPU":500,"MemoryMB":256,"DiskMB":0,"IOPS":0,"Networks":[{"Mode":"","Device":"","CIDR":"","IP":"","MBits":10,"DNS":null,"ReservedPorts":null,"DynamicPorts":[{"Label":"db","Value":0,"To":0,"HostNetwork":"default"}]}],"Devices":null},"RestartPolicy":{"Attempts":2,"Interval":1800000000000,"Delay":15000000000,"Mode":"fail"},"DispatchPayload":null,"Lifecycle":null,"Meta":null,"KillTimeout":5000000000,"LogConfig":{"MaxFiles":10,"MaxFileSizeMB":10},"Artifacts":null,"Leader":false,"ShutdownDelay":0,"VolumeMounts":null,"KillSignal":"","Kind":"","CSIPluginConfig":null}],"EphemeralDisk":{"Sticky":false,"SizeMB":300,"Migrate":false},"Meta":null,"ReschedulePolicy":{"Attempts":0,"Interval":0,"Delay":30000000000,"DelayFunction":"exponential","MaxDelay":3600000000000,"Unlimited":true},"Affinities":null,"Spreads":null,"Networks":null,"Services":null,"Volumes":null,"ShutdownDelay":null,"StopAfterClientDisconnect":null}],"Update":{"Stagger":30000000000,"MaxParallel":1,"HealthCheck":"","MinHealthyTime":0,"HealthyDeadline":0,"ProgressDeadline":0,"AutoRevert":false,"AutoPromote":false,"Canary":0},"Multiregion":null,"Periodic":null,"ParameterizedJob":null,"Dispatched":false,"Payload":null,"Meta":null,"ConsulToken":"","VaultToken":"","VaultNamespace":"","NomadTokenID":"","Status":"running","StatusDescription":"","Stable":false,"Version":0,"SubmitTime":1604360707460244478,"CreateIndex":403,"ModifyIndex":410,"JobModifyIndex":403}"#;

    const JOB_LISTING: &str = r#"[{"ID":"example","ParentID":"","Name":"example","Namespace":"","Datacenters":["dc1"],"Multiregion":null,"Type":"service","Priority":50,"Periodic":false,"ParameterizedJob":false,"Stop":false,"Status":"running","StatusDescription":"","JobSummary":{"JobID":"example","Namespace":"default","Summary":{"cache":{"Queued":0,"Complete":0,"Failed":0,"Running":1,"Starting":0,"Lost":0}},"Children":{"Pending":0,"Running":0,"Dead":0},"CreateIndex":403,"ModifyIndex":413},"CreateIndex":403,"ModifyIndex":410,"JobModifyIndex":403,"SubmitTime":1604360707460244478}]"#;

    struct TestClient {
        path: Option<String>,
        response_status_code: u16,
        response_body: &'static str,
        response_status_text: &'static str,
    }

    impl NomadClient for TestClient {
        fn get(&mut self, resource: &str) -> Result<ureq::Response> {
            self.path = Some(String::from(resource));
            Ok(ureq::Response::new(
                self.response_status_code,
                self.response_status_text,
                self.response_body,
            ))
        }
    }

    #[test]
    fn test_get_job() {
        let mut client = TestClient {
            path: None,
            response_status_code: 200,
            response_status_text: "OK",
            response_body: FULL_JOB,
        };
        let result = get_job(&mut client, "example");
        assert_eq!(client.path, Some(String::from("job/example")));
        assert!(result.is_ok());
        let job = result.unwrap();
        // For some reason, serde flatten doesn't work in test mode *shrug*
        assert_eq!(job.listing.ID, "example");
    }

    #[test]
    fn test_get_job_missing() {
        let mut client = TestClient {
            path: None,
            response_status_code: 400,
            response_status_text: "Bad Request",
            response_body: "",
        };
        let result = get_job(&mut client, "example");
        assert_eq!(client.path, Some(String::from("job/example")));
        assert!(result.is_err());
        match result {
            Err(err) => assert_eq!(err.to_string(), "failed to read response"),
            Ok(_) => unreachable!(),
        };
    }

    #[test]
    fn test_get_jobs_no_prefix() {
        let mut client = TestClient {
            path: None,
            response_status_code: 200,
            response_status_text: "OK",
            response_body: JOB_LISTING,
        };
        let result = get_jobs(&mut client, "");
        assert_eq!(client.path, Some(String::from("jobs?prefix=")));
        assert!(result.is_ok());
        let job = result.unwrap();
        // For some reason, serde flatten doesn't work in test mode *shrug*
        assert_eq!(job.len(), 1);
        assert_eq!(job[0].ID, "example");
    }

    #[test]
    fn test_get_jobs_malformed() {
        let mut client = TestClient {
            path: None,
            response_status_code: 400,
            response_status_text: "Bad Request",
            response_body: "",
        };
        let result = get_jobs(&mut client, "");
        assert_eq!(client.path, Some(String::from("jobs?prefix=")));
        assert!(result.is_err());
        // For some reason, serde flatten doesn't work in test mode *shrug*
        match result {
            Err(err) => assert_eq!(err.to_string(), "failed to read response"),
            Ok(_) => unreachable!(),
        };
    }

    #[test]
    fn test_get_jobs_with_prefix() {
        let mut client = TestClient {
            path: None,
            response_status_code: 200,
            response_status_text: "OK",
            response_body: JOB_LISTING,
        };
        let result = get_jobs(&mut client, "example");
        assert_eq!(client.path, Some(String::from("jobs?prefix=example")));
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_jobs_with_prefix_quoted() {
        let mut client = TestClient {
            path: None,
            response_status_code: 200,
            response_status_text: "OK",
            response_body: JOB_LISTING,
        };
        let result = get_jobs(&mut client, "dispatch-example/periodic-102002");
        assert_eq!(
            client.path,
            Some(String::from(
                "jobs?prefix=dispatch%2Dexample%2Fperiodic%2D102002"
            ))
        );
        assert!(result.is_ok());
    }
}
