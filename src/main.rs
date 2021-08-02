use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

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
    let mut ttspico_backend = TTSPicoBackend::new(
        memory_size,
        text_analysis_data_path,
        speech_generation_data_pata,
    )?;
    let source = b"1, 2, 3, Hello Rust!\0";
    let pcm_data = ttspico_backend.generate(source)?;
    // Plays an audio buffer (16-bit signed PCM @ 16kHz)
    // to the system default output device.
    // Exits the current process when done.
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("No sound output device");
    let config = cpal::StreamConfig {
        channels: 1,
        sample_rate: cpal::SampleRate(16_000),
        buffer_size: cpal::BufferSize::Default,
    };
    let mut buffer = pcm_data.into_iter();
    let data_sender =
        move |data: &mut cpal::Data, _: &cpal::OutputCallbackInfo| {
            data.as_slice_mut()
                .unwrap()
                .iter_mut()
                .for_each(|d| *d = buffer.next().unwrap_or(0i16))
        };
    let error_handler = |err| {
        eprintln!("Failed to output audio stream.\n{}", err);
    };
    let stream = device
        .build_output_stream_raw(
            &config,
            cpal::SampleFormat::I16,
            data_sender,
            error_handler,
        )
        .expect("Failed to build output stream.");
    stream.play().expect("Failed to play output stream.");
    std::thread::sleep(std::time::Duration::from_millis(5000));
    Ok(())
}

pub struct TTSPicoError {
    description: String,
}
impl From<ttspico::PicoError> for TTSPicoError {
    fn from(source: ttspico::PicoError) -> TTSPicoError {
        let description = source.descr;
        TTSPicoError { description }
    }
}
impl std::error::Error for TTSPicoError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
impl std::fmt::Debug for TTSPicoError {
    fn fmt(
        &self,
        formatter: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        formatter
            .debug_struct("TTSPicoError")
            .field("description", &self.description)
            .finish()
    }
}
impl std::fmt::Display for TTSPicoError {
    fn fmt(
        &self,
        formatter: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        write!(formatter, "TTS Pico error: {}", self.description)
    }
}

pub struct TTSPicoBackend {
    engine: ttspico::Engine,
}
impl TTSPicoBackend {
    pub fn new(
        memory_size: usize,
        text_analysis_data_path: &str,
        speech_generation_data_path: &str,
    ) -> Result<TTSPicoBackend, TTSPicoError> {
        let system = ttspico::System::new(memory_size)?;
        let text_analysis_data = ttspico::System::load_resource(
            std::rc::Rc::clone(&system),
            text_analysis_data_path,
        )?;
        let speech_generation_data = ttspico::System::load_resource(
            std::rc::Rc::clone(&system),
            speech_generation_data_path,
        )?;
        let voice = ttspico::System::create_voice(system, "TestVoice")?;
        voice.borrow_mut().add_resource(text_analysis_data)?;
        voice.borrow_mut().add_resource(speech_generation_data)?;
        let engine = unsafe { ttspico::Voice::create_engine(voice)? };
        Ok(TTSPicoBackend { engine })
    }

    pub fn generate(
        &mut self,
        source: &[u8],
    ) -> Result<Vec<i16>, TTSPicoError> {
        let mut remaining = source;
        while remaining.len() > 0 {
            let n_put = self.engine.put_text(remaining)?;
            remaining = &remaining[n_put..];
        }
        // 16-bit signed 16kHz PCM.
        let mut pcm_data: Vec<i16> = vec![0i16; 0];
        let mut pcm_buf = [0i16; 1024];
        loop {
            let (n_written, status) =
                self.engine.get_data(&mut pcm_buf[..])?;
            pcm_data.extend(&pcm_buf[..n_written]);
            if status == ttspico::EngineStatus::Idle {
                break;
            }
        }
        Ok(pcm_data)
    }
}
