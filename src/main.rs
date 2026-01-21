mod address;
mod assembler;
mod cli;
mod lexer;
mod registers;
mod simulator;

use cli::CLI;
use simulator::Simulator;
use std::process;

use crate::simulator::SimulatorError;

fn main() {
    let args = CLI::new();
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

    let mut assembler = assembler::Assembler::new();
    if let Err(err) = assembler.assemble(&args) {
        println!("Assembler Error: {:?}", err);
        return;
    }

    let memory = assembler.take_memory();

    if args.memory {
        println!("{:?}", memory);
    }

    let instructions = assembler.get_instructions();
    let entry = assembler.get_entry_point();

    let mut simulator = Simulator::new(instructions, memory, entry);

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
