mod chart;
mod cli;

fn main() {
    if let Err(msg) = cli::main() {
        println!("{}", msg);
        println!("{}", cli::HELP);
    }
}
