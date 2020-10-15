use jsonpath::Selector;
use log::trace;
use std::collections::HashMap;
use structopt::StructOpt;

mod nomad;

#[derive(Debug, StructOpt)]
#[structopt(name = "nquery")]
struct Opt {
    #[structopt(long)]
    status: Option<String>,

    #[structopt(long, conflicts_with = "no_periodic")]
    periodic: bool,

    #[structopt(long, conflicts_with = "periodic")]
    no_periodic: bool,

    #[structopt(long, conflicts_with = "no_parameterized")]
    parameterized: bool,

    #[structopt(long, conflicts_with = "parameterized")]
    no_parameterized: bool,

    #[structopt(long)]
    pretty: bool,

    #[structopt(long = "type")]
    job_type: Option<String>,

    #[structopt(short, long, number_of_values = 1)]
    fields: Vec<String>,

    #[structopt(default_value = "")]
    job_name: String,
}

fn get_jobs(
    name_filter: &str,
    status_filter: Option<String>,
    job_type_filter: Option<String>,
    periodic_filter: Option<bool>,
    parameterized_filter: Option<bool>,
) -> Vec<nomad::Job> {
    let client = nomad::get_client();
    let server = nomad::Nomad { client };
    let job_listing = server.get_jobs().unwrap();
    job_listing
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
        .collect()
}

fn handle_negative_flags(flag_tuple: (bool, bool)) -> Option<bool> {
    match flag_tuple {
        (false, false) => None,
        (true, false) => Some(true),
        (false, true) => Some(false),
        (_, _) => panic!("This shouldn't happen"),
    }
}

fn build_paths(fields: Vec<String>) -> HashMap<String, Selector> {
    let paths: HashMap<String, Selector> = fields
        .iter()
        .map(|s| (String::from(s), Selector::new(&format!("$.{}", s)).unwrap()))
        .collect();
    paths
}

fn main() {
    env_logger::init();
    let cmd = Opt::from_args();
    let periodic = handle_negative_flags((cmd.periodic, cmd.no_periodic));
    let parameterized = handle_negative_flags((cmd.parameterized, cmd.no_parameterized));
    let jobs: Vec<nomad::Job> = get_jobs(
        &cmd.job_name,
        cmd.status,
        cmd.job_type,
        periodic,
        parameterized,
    );
    let mut flattened = serde_json::to_value(&jobs).unwrap();
    if !(&cmd.fields).is_empty() {
        let paths = build_paths(cmd.fields);
        let mut full_jobs: Vec<serde_json::Value> = Vec::new();
        for job in jobs {
            let mut job_view: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
            for (path, selector) in &paths {
                let job_json = serde_json::to_value(&job).unwrap();
                let matches: Vec<&serde_json::Value> = selector.find(&job_json).collect();
                if !matches.is_empty() {
                    for matched in matches {
                        trace!("Match: {}, {}", path, matched);
                        job_view.insert(path.to_string(), matched.to_owned());
                    }
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
