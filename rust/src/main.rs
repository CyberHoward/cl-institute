use clap::Parser;

#[derive(Parser)]
#[command(name = "clinstitute", version, about = "clinstitute CLI")]
struct Cli {}

fn main() {
    let _cli = Cli::parse();
}
