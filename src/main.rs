mod cpal;
mod ttspico;

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
    let memory_size = 4 * 1024 * 1024;
    let text_analysis_data_path = "/usr/share/pico/lang/en-GB_ta.bin";
    let speech_generation_data_pata =
        "/usr/share/pico/lang/en-GB_kh0_sg.bin";
    let mut ttspico_backend = ttspico::TTSPicoBackend::new(
        memory_size,
        text_analysis_data_path,
        speech_generation_data_pata,
    )?;
    let source = b"1, 2, 3, Hello Rust!\0";
    // An audio buffer (16-bit signed PCM @ 16kHz).
    let pcm_data = ttspico_backend.generate(source)?;
    let cpal_output = cpal::Output::new()?;
    cpal_output.queue(pcm_data)?;
    Ok(())
}
