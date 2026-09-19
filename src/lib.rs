use nice_plug::prelude::*;
use std::sync::Arc;

pub struct Cie {
    params: Arc<CieParams>,
    buffer: Vec<f32>,
    buffer_pointer: usize,
    sample_rate: f32,
}

#[derive(Params)]
struct CieParams {
    #[id = "gain"]
    pub gain: FloatParam,
    #[id = "delay"]
    pub delay: FloatParam,
    #[id = "feedback"]
    pub feedback: FloatParam,
}

impl Default for CieParams {
    fn default() -> Self {
        Self {
            gain: FloatParam::new(
                "Gain",
                util::db_to_gain(0.0),
                FloatRange::gain_range(-30.0, 30.0),
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),
            delay: FloatParam::new("Delay", 0.0, FloatRange::Linear { min: 0.0, max: 5.0 })
                .with_unit("S"), // This might need to be longer, 5 seems good for testing now, probably shoudl be in ms as well
            feedback: FloatParam::new("Feedback", 0.5, FloatRange::Linear { min: 0.0, max: 1.0 }) // Obviously multiply this to make it really percentage
                .with_unit(" %"),
        }
    }
}

impl Default for Cie {
    fn default() -> Self {
        Self {
            params: Arc::new(CieParams::default()),
            buffer_pointer: 0,
            sample_rate: 44100.0,
            buffer: Vec::new(),
        }
    }
}

impl Plugin for Cie {
    const NAME: &'static str = "Cie";
    const VENDOR: &'static str = "G-Stpierre";
    const URL: &'static str = "https://github.com/G-StPierre/Cie";
    const EMAIL: &'static str = "info@example.com";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(2),
            main_output_channels: NonZeroU32::new(2),
            aux_input_ports: &[],
            aux_output_ports: &[],
            names: PortNames::const_default(),
        },
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(1),
            main_output_channels: NonZeroU32::new(1),
            ..AudioIOLayout::const_default()
        },
    ];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type Editor = ();
    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn activate(
        &mut self,
        audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        context: &mut impl ActivateContext<Self>,
    ) -> bool {
        let sample_rate = buffer_config.sample_rate as usize;

        self.buffer = vec![0.0; sample_rate * 5 * 2]; // Agian my 5 seconds max and both left and right ear so x2

        self.sample_rate = buffer_config.sample_rate;

        true
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let delay_sample = self.sample_rate * self.params.delay.value();

        let feedback = self.params.feedback.value();

        let length = self.buffer.len();

        let mut pointer = (self.buffer_pointer as isize) - (delay_sample as isize);

        if pointer < 0 {
            pointer += length as isize;
        }

        let pointer = pointer as usize;

        for channel_samples in buffer.iter_samples() {
            let gain = self.params.gain.smoothed.next();

            for sample in channel_samples {
                self.buffer[self.buffer_pointer] = *sample;
                // *sample *= gain;
                *sample += (self.buffer[pointer] * feedback);

                self.buffer_pointer += 1;
                if self.buffer_pointer >= length {
                    self.buffer_pointer = 0;
                }
            }
        }

        ProcessStatus::Normal
    }

    fn deactivate(&mut self) {}
}

impl ClapPlugin for Cie {
    const CLAP_ID: &'static str = "com.github.g-stpierre.cie";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("Pulsing audio delay");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Stereo];
}

impl Vst3Plugin for Cie {
    const VST3_CLASS_ID: [u8; 16] = *b"g-stpierre.plcie";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Tools];
}

nice_export_clap!(Cie);
nice_export_vst3!(Cie);
