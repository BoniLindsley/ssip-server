use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ttspico as pico;

fn main() {
    // 1. Create a Pico system
    // NOTE: There should at most one System per thread!
    let sys = pico::System::new(4 * 1024 * 1024)
        .expect("Could not init system");

    // 2. Load Text Analysis (TA) and Speech Generation (SG) resources
    // for the voice you want to use
    let ta_res = pico::System::load_resource(
        std::rc::Rc::clone(&sys),
        "/usr/share/pico/lang/en-GB_ta.bin",
    )
    .expect("Failed to load TA");
    let sg_res = pico::System::load_resource(
        std::rc::Rc::clone(&sys),
        "/usr/share/pico/lang/en-GB_kh0_sg.bin",
    )
    .expect("Failed to load SG");
    println!(
        "TA: {}, SG: {}",
        ta_res.borrow().name().unwrap(),
        sg_res.borrow().name().unwrap()
    );

    // 3. Create a Pico voice definition
    // and attach the loaded resources to it
    let voice = pico::System::create_voice(sys, "TestVoice")
        .expect("Failed to create voice");
    voice
        .borrow_mut()
        .add_resource(ta_res)
        .expect("Failed to add TA to voice");
    voice
        .borrow_mut()
        .add_resource(sg_res)
        .expect("Failed to add SG to voice");

    // 4. Create an engine from the voice definition
    // UNSAFE: Creating an engine without attaching the resources
    // will result in a crash!
    let mut engine = unsafe {
        pico::Voice::create_engine(voice)
            .expect("Failed to create engine")
    };

    // 5. Put (UTF-8) text to be spoken into the engine
    // See `Engine::put_text()` for more details.
    // The null terminator tells Pico to start synthesizing.
    let mut text_bytes: &[u8] = b"1, 2, 3, Hello Rust!\0";
    while text_bytes.len() > 0 {
        let n_put = engine
            .put_text(text_bytes)
            .expect("pico_putTextUtf8 failed");
        text_bytes = &text_bytes[n_put..];
    }

    // 6. Do the actual text-to-speech,
    // getting audio data (16-bit signed PCM @ 16kHz) from the input text
    // Speech audio is computed in small chunks, one "step" at a time;
    // see `Engine::get_data()` for more details.
    let mut pcm_data: Vec<i16> = vec![0i16; 0];
    let mut pcm_buf = [0i16; 1024];
    'tts: loop {
        let (n_written, status) = engine
            .get_data(&mut pcm_buf[..])
            .expect("pico_getData error");
        pcm_data.extend(&pcm_buf[..n_written]);
        if status == ttspico::EngineStatus::Idle {
            break 'tts;
        }
    }

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
    //let copied_data = pcm_data.leak();
    //let mut remaining_data = &*copied_data;

    //let data_sender =
    //    move |data: &mut cpal::Data, _: &cpal::OutputCallbackInfo| {
    //        let n_to_copy =
    //            std::cmp::min(data.len(), remaining_data.len());
    //        let data_slice =
    //            data.as_slice_mut().expect("Unmatched sample size");
    //        data_slice[..n_to_copy]
    //            .copy_from_slice(&remaining_data[..n_to_copy]);
    //        remaining_data = &remaining_data[n_to_copy..];
    //        if remaining_data.is_empty() {
    //            std::process::exit(0);
    //        }
    //    };
    let (input, mut output) = rodio::dynamic_mixer::mixer::<i16>(
        config.channels,
        config.sample_rate.0,
    );
    //let input = std::sync::Arc::new(
    //    rodio::dynamic_mixer::DynamicMixerController {
    //        has_pending: std::sync::atomic::AtomicBool::new(false),
    //        pending_sources: std::sync::Mutex::new(Vec::new()),
    //        channels: config.channels,
    //        sample_rate: config.sample_rate.0,
    //    },
    //);

    //let mut output = rodio::dynamic_mixer::DynamicMixer {
    //    current_sources: Vec::with_capacity(16),
    //    input: input.clone(),
    //    //sample_count: 0,
    //    //still_pending: vec![],
    //    //still_current: vec![],
    //};
    let buffer = rodio::buffer::SamplesBuffer::new(
        config.channels,
        config.sample_rate.0,
        pcm_data,
    );
    input.add(buffer);
    let data_sender =
        move |data: &mut cpal::Data, _: &cpal::OutputCallbackInfo| {
            data.as_slice_mut().unwrap().iter_mut().for_each(|d| {
                *d = output
                    .next() /*map(|s| s.to_i16()).*/
                    .unwrap_or(0i16)
            })
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
}
