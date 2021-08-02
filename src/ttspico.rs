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
