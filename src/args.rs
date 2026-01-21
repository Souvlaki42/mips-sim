use std::env;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Args {
    pub file: String,
    pub args: bool,
    pub help: bool,
    pub tokens: bool,
    pub instructions: bool,
    pub version: bool,
    pub memory: bool,
}

impl Args {
    pub fn new() -> Self {
        let args: Vec<String> = env::args().collect();
        return Self {
            file: match args.get(1) {
                Some(file) => file.to_string(),
                None => "".to_string(),
            },
            args: args.contains(&"-a".to_string()) || args.contains(&"--args".to_string()),
            help: args.contains(&"-h".to_string())
                || args.contains(&"--help".to_string())
                || args.get(1).is_none(),
            tokens: args.contains(&"-t".to_string()) || args.contains(&"--tokens".to_string()),
            instructions: args.contains(&"-i".to_string())
                || args.contains(&"--instructions".to_string()),
            version: args.contains(&"-v".to_string()) || args.contains(&"--version".to_string()),
            memory: args.contains(&"-m".to_string()) || args.contains(&"--memory".to_string()),
        };
    }

    pub fn print_help(&self, package_name: &str) {
        println!("Usage: {} <file> [options]", package_name);
        println!("Options:");
        println!("  -h, --help     Print this help message");
        println!("  -a, --args     Print the arguments");
        println!("  -t, --tokens   Print the tokens");
        println!("  -i, --instructions   Print the instructions");
        println!("  -m, --memory   Print the memory");
        println!("  -v, --version  Print program version");
    }
}
