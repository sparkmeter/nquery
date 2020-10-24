use anyhow::Result;

extern crate jsonpath_lib as jsonpath;
use log::trace;
use std::collections::HashMap;
use std::process;
use structopt::StructOpt;

mod nomad;

#[derive(Debug, StructOpt)]
#[structopt(name = "nquery")]
struct Opt {
    /// Return jobs with this status
    #[structopt(long)]
    status: Option<String>,

    /// Return periodic jobs
    #[structopt(long, conflicts_with = "no_periodic")]
    periodic: bool,

    /// Exclude periodic jobs
    #[structopt(long, conflicts_with = "periodic")]
    no_periodic: bool,

    /// Return parameterized jobs
    #[structopt(long, conflicts_with = "no_parameterized")]
    parameterized: bool,

    /// Exclude parameterized jobs
    #[structopt(long, conflicts_with = "parameterized")]
    no_parameterized: bool,

    /// Pretty print the JSON output
    #[structopt(long)]
    pretty: bool,

    /// Return jobs of this type
    #[structopt(long = "type")]
    job_type: Option<String>,

    /// Include only these fields in the ouput
    #[structopt(short, long, number_of_values = 1)]
    fields: Vec<String>,

    /// A prefix that the job name must match
    #[structopt(default_value = "")]
    job_name: String,
}

/// Get all jobs matching the supplied criteria
///
/// # Arguments
///
/// * `name_filter` - A string prefix that all job IDs must match
/// * `status_filter` - If specified, all jobs must have a status equal to this
/// * `job_type_filter` - If specified, all jobs must be of this type
/// * `periodic_filter` - If specified, all jobs must be either periodic or not periodic
/// * `parameterized_filter` - If specified, all jobs must be either parameterized or
/// non-parameterized.
fn get_jobs(
    name_filter: &str,
    status_filter: Option<String>,
    job_type_filter: Option<String>,
    periodic_filter: Option<bool>,
    parameterized_filter: Option<bool>,
) -> Result<Vec<nomad::Job>> {
    let client = nomad::get_client();
    let server = nomad::Nomad { client };
    let job_listing = server.get_jobs()?;
    Ok(job_listing
        .into_iter()
        .filter(|job| match periodic_filter {
            Some(is_periodic) => is_periodic == job.Periodic.unwrap_or(false),
            None => true,
        })
        .filter(|job| match parameterized_filter {
            Some(is_parameterized) => is_parameterized == job.ParameterizedJob.unwrap_or(false),
            None => true,
        })
        .filter(|job| {
            job.ID
                .to_lowercase()
                .starts_with(&name_filter.to_lowercase())
        })
        .filter(|job| match &status_filter {
            Some(status) => job.Status.eq_ignore_ascii_case(&status),
            None => true,
        })
        .filter(|job| match &job_type_filter {
            Some(job_type) => job.Type.eq_ignore_ascii_case(&job_type),
            None => true,
        })
        .map(|job| server.get_job(&job.ID).unwrap())
        .map(|job| {
            trace!("Individual Job: {:#?}", job);
            job
        })
        .collect())
}

/// Build a ternary value from a combination of boolean values.
///
/// # Arguments
///
/// * `flag_tuple` - A tuple of boolean values, the first being the positive, and the second being
/// the negative
///
/// # Examples
///
/// ```
/// assert_eq!(Some(true), handle_negative_flags((true, false)))
/// assert_eq!(None, handle_negative_flags((false, false)))
/// assert_eq!(Some(false), handle_negative_flags((false, true)))
/// ```
fn handle_negative_flags(flag_tuple: (bool, bool)) -> Option<bool> {
    match flag_tuple {
        (false, false) => None,
        (true, false) => Some(true),
        (false, true) => Some(false),
        (_, _) => panic!("This shouldn't happen"),
    }
}

/// Run the thing!
fn main() {
    env_logger::init();
    color_backtrace::install();
    let cmd = Opt::from_args();
    let periodic = handle_negative_flags((cmd.periodic, cmd.no_periodic));
    let parameterized = handle_negative_flags((cmd.parameterized, cmd.no_parameterized));
    let jobs: Vec<nomad::Job> = match get_jobs(
        &cmd.job_name,
        cmd.status,
        cmd.job_type,
        periodic,
        parameterized,
    ) {
        Ok(found_jobs) => found_jobs,
        Err(err) => {
            eprintln!("{}", err.to_string());
            process::exit(1);
        }
    };
    let mut flattened = serde_json::to_value(&jobs).unwrap();
    if !(&cmd.fields).is_empty() {
        let paths: HashMap<String, String> = cmd
            .fields
            .iter()
            .map(|f| (String::from(f), format!("$.{}", f)))
            .collect();
        let mut full_jobs: Vec<serde_json::Value> = Vec::new();
        for job in jobs {
            let mut job_view: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
            for (path, selector) in &paths {
                let job_json = serde_json::to_value(&job).unwrap();
                let matches: Vec<&serde_json::Value> =
                    jsonpath::select(&job_json, selector).unwrap();
                for matched in matches {
                    trace!("Match: {}, {}", path, matched);
                    job_view.insert(path.to_string(), matched.to_owned());
                }
            }
            full_jobs.push(serde_json::to_value(job_view).unwrap());
        }
        flattened = serde_json::to_value(full_jobs).unwrap();
    }
    if cmd.pretty {
        println!("{}", serde_json::to_string_pretty(&flattened).unwrap());
    } else {
        println!("{}", serde_json::to_string(&flattened).unwrap());
    }
}
