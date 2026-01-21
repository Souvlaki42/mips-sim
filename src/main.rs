mod address;
mod args;
mod assembler;
mod lexer;
mod registers;
mod simulator;

use args::Args;
use simulator::Simulator;
use std::{collections::HashMap, process};

use crate::simulator::SimulatorError;

fn main() {
    let args = Args::new();
    let package_name = env!("CARGO_PKG_NAME");
    let package_version = env!("CARGO_PKG_VERSION");

    if args.version {
        println!("{} v{}", package_name, package_version);
        return;
    }

    if args.help {
        args.print_help(package_name);
        return;
    }

    if args.args {
        println!("{:?}", args);
    }

    let mut memory = HashMap::new();

    let mut assembler = assembler::Assembler::new(&mut memory);
    if let Err(err) = assembler.assemble(&args) {
        println!("Assembler Error: {:?}", err);
        return;
    }

    let instructions = assembler.get_instructions();
    let entry = assembler.get_entry_point();

    let mut simulator = Simulator::new(instructions, &mut memory, entry);

    let mut exit_code = 0;
    loop {
        if let Err(err) = simulator.step() {
            match err {
                SimulatorError::Exit(value) => {
                    exit_code = value as i32;
                    println!("\n-- program is finished running --");
                }
                SimulatorError::NoMoreInstructions => {
                    println!("\n-- program is finished running (dropped off bottom) --");
                }
                _ => println!("Simulator Error: {:?}", err),
            }
            break;
        }
    }
    process::exit(exit_code);
}
