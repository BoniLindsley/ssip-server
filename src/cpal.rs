use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

pub struct Error {
    description: String,
}
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
impl std::fmt::Debug for Error {
    fn fmt(
        &self,
        formatter: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        formatter
            .debug_struct("Error")
            .field("description", &self.description)
            .finish()
    }
}
impl std::fmt::Display for Error {
    fn fmt(
        &self,
        formatter: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        write!(formatter, "TTS Pico error: {}", self.description)
    }
}

pub struct Output {
    device: <cpal::Host as HostTrait>::Device,
    config: cpal::StreamConfig,
}
impl Output {
    pub fn new() -> Result<Output, Error> {
        let host: cpal::Host = cpal::default_host();
        let device: <cpal::Host as HostTrait>::Device =
            host.default_output_device().ok_or_else(|| Error {
                description: String::from(
                    "Unable to find audio output device.",
                ),
            })?;
        let config = cpal::StreamConfig {
            channels: 1,
            sample_rate: cpal::SampleRate(16_000),
            buffer_size: cpal::BufferSize::Default,
        };
        Ok(Output { device, config })
    }
    pub fn play(
        &self,
        source: Vec<i16>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut buffer = source.into_iter();
        let data_sender =
            move |data: &mut cpal::Data,
                  _info: &cpal::OutputCallbackInfo| {
                data.as_slice_mut()
                    .unwrap()
                    .iter_mut()
                    .for_each(|d| *d = buffer.next().unwrap_or(0i16))
            };
        let error_handler = |error: cpal::StreamError| {
            eprintln!("Failed to output audio stream.\n{}", error);
        };
        let stream = self
            .device
            .build_output_stream_raw(
                &self.config,
                cpal::SampleFormat::I16,
                data_sender,
                error_handler,
            )?;
        stream.play()?;
        std::thread::sleep(std::time::Duration::from_millis(5000));
        Ok(())
    }
}
