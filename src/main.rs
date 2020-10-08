use structopt::StructOpt;

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

    #[structopt(long = "type")]
    job_type: Option<String>,

    #[structopt(short, long, number_of_values = 1)]
    fields: Vec<String>,

    job_name: String,
}

fn handle_negative_flags(flag_tuple: (bool, bool)) -> Option<bool> {
    match flag_tuple {
        (false, false) => None,
        (true, false) => Some(true),
        (false, true) => Some(false),
        (_, _) => panic!("This shouldn't happen"),
    }
}

fn main() {
    let cmd = Opt::from_args();
    let periodic = handle_negative_flags((cmd.periodic, cmd.no_periodic));
    let parameterized = handle_negative_flags((cmd.parameterized, cmd.no_parameterized));
    println!("{:#?}, {:#?}, {:#?}", cmd, periodic, parameterized);
}
