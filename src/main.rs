use clap::Parser;

mod commands;

fn main() {
    let args = commands::ls::Ls::parse();

    commands::ls::ls(args).unwrap();
}
