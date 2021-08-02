use ssip_server;

type ExitCode = i32;
type RunResult = Result<(), Box<dyn std::error::Error>>;

fn main() {
    let run_result = run();
    let exit_code = parse_run_result(run_result);
    std::process::exit(exit_code);
}

fn parse_run_result(result: RunResult) -> ExitCode {
    match result {
        Ok(()) => 0,
        Err(error) => {
            eprintln!("{}", error);
            1
        }
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut output_module = ssip_server::OutputModule::new()?;
    output_module.speak("1, 2, 3, Hello Rust!\0")?;
    Ok(())
}
