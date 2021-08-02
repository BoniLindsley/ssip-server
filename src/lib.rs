mod cpal;
mod ttspico;

pub struct OutputModule {
    ttspico_synthesiser: ttspico::Synthesiser,
    cpal_output: cpal::Output,
}
impl OutputModule {
    pub fn new() -> Result<OutputModule, Box<dyn std::error::Error>> {
        let ttspico_synthesiser = ttspico::Synthesiser::new()?;
        let cpal_output = cpal::Output::new()?;
        Ok(OutputModule {
            ttspico_synthesiser,
            cpal_output,
        })
    }
    pub fn speak(
        &mut self,
        text: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let source = text.as_bytes();
        let pcm_data = self.ttspico_synthesiser.generate(source)?;
        self.cpal_output.play(pcm_data)?;
        Ok(())
    }
}
