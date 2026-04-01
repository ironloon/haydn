/// Schroeder algorithmic reverb — simulates room acoustics.
///
/// Uses 4 parallel comb filters feeding into 2 series allpass filters.
/// This is a classic design that creates a dense, natural-sounding reverb
/// tail without loading external impulse response files.

/// Comb filter: delayed feedback loop that creates resonant echoes.
struct CombFilter {
    buffer: Vec<f32>,
    index: usize,
    feedback: f32,
    /// Simple low-pass damping in the feedback path (models air absorption)
    damp: f32,
    prev: f32,
}

impl CombFilter {
    fn new(delay_samples: usize, feedback: f32, damp: f32) -> Self {
        Self {
            buffer: vec![0.0; delay_samples],
            index: 0,
            feedback,
            damp,
            prev: 0.0,
        }
    }

    fn process(&mut self, input: f32) -> f32 {
        let delayed = self.buffer[self.index];
        // Low-pass filter in feedback path: damp controls how much high freq is absorbed
        self.prev = delayed * (1.0 - self.damp) + self.prev * self.damp;
        self.buffer[self.index] = input + self.prev * self.feedback;
        self.index = (self.index + 1) % self.buffer.len();
        delayed
    }
}

/// Allpass filter: passes all frequencies equally but adds phase dispersion,
/// which thickens the reverb tail and prevents metallic ringing.
struct AllpassFilter {
    buffer: Vec<f32>,
    index: usize,
    feedback: f32,
}

impl AllpassFilter {
    fn new(delay_samples: usize, feedback: f32) -> Self {
        Self {
            buffer: vec![0.0; delay_samples],
            index: 0,
            feedback,
        }
    }

    fn process(&mut self, input: f32) -> f32 {
        let delayed = self.buffer[self.index];
        let output = -input + delayed;
        self.buffer[self.index] = input + delayed * self.feedback;
        self.index = (self.index + 1) % self.buffer.len();
        output
    }
}

/// Room reverb processor.
///
/// Apply to a mono buffer in-place. Uses mutually-prime delay lengths
/// to avoid resonant peaks, and damping to simulate air absorption
/// (high frequencies die faster than lows, like a real room).
pub struct Reverb {
    combs: Vec<CombFilter>,
    allpasses: Vec<AllpassFilter>,
    wet: f32,
    dry: f32,
}

impl Reverb {
    /// Create a reverb tuned for the given sample rate.
    ///
    /// - `mix`: 0.0 = fully dry, 1.0 = fully wet. 0.15-0.25 is natural for most music.
    /// - `room_size`: 0.0 = small room, 1.0 = large hall. Scales feedback.
    /// - `damping`: 0.0 = bright reverb, 1.0 = very dark. 0.4-0.6 is natural.
    pub fn new(sample_rate: u32, mix: f32, room_size: f32, damping: f32) -> Self {
        let sr_scale = sample_rate as f32 / 44100.0;

        // Mutually-prime delay lengths in samples (at 44.1kHz, scaled for other rates).
        // These are classic Schroeder values adjusted for musicality.
        let comb_delays = [1557, 1617, 1491, 1422, 1277, 1356, 1188, 1116];
        let allpass_delays = [225, 556, 441, 341];

        let base_feedback = 0.7 + room_size.clamp(0.0, 1.0) * 0.28; // 0.70 to 0.98
        let damp = damping.clamp(0.0, 1.0);

        let combs = comb_delays
            .iter()
            .map(|&d| {
                let delay = ((d as f32) * sr_scale) as usize;
                CombFilter::new(delay, base_feedback, damp)
            })
            .collect();

        let allpasses = allpass_delays
            .iter()
            .map(|&d| {
                let delay = ((d as f32) * sr_scale) as usize;
                AllpassFilter::new(delay, 0.5)
            })
            .collect();

        Self {
            combs,
            allpasses,
            wet: mix.clamp(0.0, 1.0),
            dry: 1.0 - mix.clamp(0.0, 1.0),
        }
    }

    /// Concert hall preset — spacious with moderate damping.
    pub fn hall(sample_rate: u32) -> Self {
        Self::new(sample_rate, 0.18, 0.8, 0.45)
    }

    /// Process a mono buffer in-place, adding reverb.
    pub fn process(&mut self, buffer: &mut [f32]) {
        for sample in buffer.iter_mut() {
            let input = *sample;

            // Sum parallel comb filter outputs
            let comb_sum: f32 = self.combs.iter_mut().map(|c| c.process(input)).sum();
            let mut wet = comb_sum / self.combs.len() as f32;

            // Series allpass filters
            for ap in &mut self.allpasses {
                wet = ap.process(wet);
            }

            *sample = input * self.dry + wet * self.wet;
        }
    }
}

/// Convert a mono buffer to interleaved stereo with subtle width.
///
/// Creates stereo by applying a tiny L/R delay difference (Haas effect)
/// and slight decorrelation, giving the impression of space without
/// making the sound feel off-center.
pub fn mono_to_stereo(mono: &[f32], sample_rate: u32) -> Vec<f32> {
    // Haas delay: ~0.6ms offset between L and R
    let delay_samples = (0.0006 * sample_rate as f64) as usize;
    let len = mono.len();
    let mut stereo = vec![0.0f32; len * 2];

    for i in 0..len {
        let left = mono[i];
        // Right channel is slightly delayed — creates width perception
        let right = if i >= delay_samples {
            mono[i - delay_samples]
        } else {
            0.0
        };

        // Slight cross-feed for natural stereo (not hard-panned)
        let cross = 0.15;
        stereo[i * 2] = left * (1.0 - cross) + right * cross;
        stereo[i * 2 + 1] = right * (1.0 - cross) + left * cross;
    }

    stereo
}

/// Soft saturation — warm analog-style limiting.
///
/// Instead of hard clipping at ±1.0, the signal is gently compressed
/// using a tanh curve. This adds subtle harmonic warmth and prevents
/// harsh digital distortion when notes stack up.
pub fn soft_saturate(buffer: &mut [f32], drive: f32) {
    let d = drive.max(1.0);
    for sample in buffer.iter_mut() {
        // tanh provides smooth saturation; normalize so small signals pass through
        *sample = (*sample * d).tanh();
    }
}

/// One-pole lowpass filter — used for damping and bandwidth control in DattorroReverb.
struct OnePole {
    coeff: f32,
    prev: f32,
}

impl OnePole {
    fn new(coeff: f32) -> Self {
        Self { coeff, prev: 0.0 }
    }

    fn process(&mut self, input: f32) -> f32 {
        self.prev = input * (1.0 - self.coeff) + self.prev * self.coeff;
        self.prev
    }
}

/// Allpass filter for the Dattorro plate reverb topology.
struct DattorroAllpass {
    buffer: Vec<f32>,
    index: usize,
    feedback: f32,
}

impl DattorroAllpass {
    fn new(delay_samples: usize, feedback: f32) -> Self {
        Self {
            buffer: vec![0.0; delay_samples.max(1)],
            index: 0,
            feedback,
        }
    }

    fn process(&mut self, input: f32) -> f32 {
        let delayed = self.buffer[self.index];
        let output = -input * self.feedback + delayed;
        self.buffer[self.index] = input + delayed * self.feedback;
        self.index = (self.index + 1) % self.buffer.len();
        output
    }
}

/// Dattorro plate reverb — smoother, more natural reverb than Schroeder.
///
/// Based on Jon Dattorro's "Effect Design Part 1" (JAES, 1997).
/// Uses a figure-8 tank topology with input diffusion, decay diffusion,
/// damping, and LFO-modulated delay lines for lush, dense reverb tails.
pub struct DattorroReverb {
    // Input section
    bandwidth_filter: OnePole,
    input_diffusors: [DattorroAllpass; 4],
    // Tank (two halves)
    decay_diffusor_1: DattorroAllpass,
    delay_1: Vec<f32>,
    delay_1_idx: usize,
    delay_1_len: usize,
    damping_1: OnePole,
    decay_diffusor_2: DattorroAllpass,
    delay_2: Vec<f32>,
    delay_2_idx: usize,
    delay_2_len: usize,
    damping_2: OnePole,
    // Modulation
    mod_lfo_phase: f32,
    mod_lfo_rate: f32,
    mod_excursion: usize,
    // Parameters
    decay: f32,
    wet: f32,
    dry: f32,
    // Tank cross-feed state
    tank_1_feedback: f32,
    tank_2_feedback: f32,
    sample_rate: u32,
}

impl DattorroReverb {
    /// Create a Dattorro plate reverb.
    ///
    /// - `sample_rate`: audio sample rate
    /// - `mix`: 0.0 = fully dry, 1.0 = fully wet
    /// - `decay`: 0.0 = short tail, 1.0 = infinite (dangerous). 0.5–0.85 is musical.
    /// - `damping`: 0.0 = bright, 1.0 = very dark. Controls high-frequency absorption.
    pub fn new(sample_rate: u32, mix: f32, decay: f32, damping: f32) -> Self {
        let sr_scale = sample_rate as f64 / 29761.0;
        let scale = |len: usize| -> usize { ((len as f64) * sr_scale).max(1.0) as usize };

        // Input diffusors (4 allpass filters in series)
        let input_diffusors = [
            DattorroAllpass::new(scale(142), 0.75),
            DattorroAllpass::new(scale(107), 0.75),
            DattorroAllpass::new(scale(379), 0.625),
            DattorroAllpass::new(scale(277), 0.625),
        ];

        // Tank components
        let decay_diffusor_1 = DattorroAllpass::new(scale(672), 0.70);
        let delay_1_len = scale(4453);
        let decay_diffusor_2 = DattorroAllpass::new(scale(1800), 0.50);
        let delay_2_len = scale(3720);

        let mod_excursion = scale(16);

        Self {
            bandwidth_filter: OnePole::new(1.0 - 0.9995_f32),
            input_diffusors,
            decay_diffusor_1,
            delay_1: vec![0.0; delay_1_len + mod_excursion + 1],
            delay_1_idx: 0,
            delay_1_len,
            damping_1: OnePole::new(damping.clamp(0.0, 1.0)),
            decay_diffusor_2,
            delay_2: vec![0.0; delay_2_len + mod_excursion + 1],
            delay_2_idx: 0,
            delay_2_len,
            damping_2: OnePole::new(damping.clamp(0.0, 1.0)),
            mod_lfo_phase: 0.0,
            mod_lfo_rate: 1.0 / sample_rate as f32, // 1 Hz LFO
            mod_excursion,
            decay: decay.clamp(0.0, 0.99),
            wet: mix.clamp(0.0, 1.0),
            dry: 1.0 - mix.clamp(0.0, 1.0),
            tank_1_feedback: 0.0,
            tank_2_feedback: 0.0,
            sample_rate,
        }
    }

    /// Hall preset — spacious plate reverb.
    pub fn hall(sample_rate: u32) -> Self {
        Self::new(sample_rate, 0.20, 0.70, 0.45)
    }

    /// Process a mono buffer in-place, adding plate reverb.
    pub fn process(&mut self, buffer: &mut [f32]) {
        let d1_total = self.delay_1.len();
        let d2_total = self.delay_2.len();

        for sample in buffer.iter_mut() {
            let input = *sample;

            // 1. Bandwidth filter
            let mut x = self.bandwidth_filter.process(input);

            // 2. Input diffusion (4 allpass filters in series)
            for d in &mut self.input_diffusors {
                x = d.process(x);
            }

            // 3. Tank: Half 1 (input + feedback from half 2)
            let t1_in = x + self.tank_2_feedback;
            let t1_ap = self.decay_diffusor_1.process(t1_in);

            // Write to delay 1
            self.delay_1[self.delay_1_idx] = t1_ap;

            // Read with LFO modulation (linear interpolation for smooth modulation)
            let mod_val = (self.mod_lfo_phase * std::f32::consts::TAU).sin();
            let mod_samples = mod_val * self.mod_excursion as f32;
            let actual_delay = (self.delay_1_len as f32 - mod_samples).max(1.0);
            let read_f = (self.delay_1_idx as f32 - actual_delay + d1_total as f32)
                % d1_total as f32;
            let read_f = read_f.max(0.0);
            let idx0 = (read_f as usize) % d1_total;
            let idx1 = (idx0 + 1) % d1_total;
            let frac = read_f - read_f.floor();
            let t1_out = self.delay_1[idx0] * (1.0 - frac) + self.delay_1[idx1] * frac;

            let t1_damped = self.damping_1.process(t1_out);
            self.tank_1_feedback = t1_damped * self.decay;

            // Tank: Half 2 (input + feedback from half 1)
            let t2_in = x + self.tank_1_feedback;
            let t2_ap = self.decay_diffusor_2.process(t2_in);

            // Write to delay 2
            self.delay_2[self.delay_2_idx] = t2_ap;

            // Read (no modulation for asymmetry)
            let read_pos = if self.delay_2_idx >= self.delay_2_len {
                self.delay_2_idx - self.delay_2_len
            } else {
                d2_total + self.delay_2_idx - self.delay_2_len
            };
            let t2_out = self.delay_2[read_pos % d2_total];

            let t2_damped = self.damping_2.process(t2_out);
            self.tank_2_feedback = t2_damped * self.decay;

            // 4. Output — sum from both tank halves
            let wet_out = (t1_out + t2_out) * 0.5;

            // Advance delay write positions
            self.delay_1_idx = (self.delay_1_idx + 1) % d1_total;
            self.delay_2_idx = (self.delay_2_idx + 1) % d2_total;

            // Advance LFO
            self.mod_lfo_phase += self.mod_lfo_rate;
            if self.mod_lfo_phase >= 1.0 {
                self.mod_lfo_phase -= 1.0;
            }

            *sample = input * self.dry + wet_out * self.wet;
        }
    }
}

/// Second-order IIR biquad filter configured as a peaking EQ.
/// Each band boosts or cuts a frequency region.
pub struct BiquadEq {
    // Coefficients
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    // State
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

impl BiquadEq {
    /// Create a peaking EQ filter.
    /// - `freq_hz`: center frequency
    /// - `gain_db`: boost/cut in dB (positive = boost, negative = cut). Clamped to ±6dB.
    /// - `q`: bandwidth (higher Q = narrower band). Typical: 0.5–2.0.
    /// - `sample_rate`: audio sample rate
    pub fn peaking(freq_hz: f32, gain_db: f32, q: f32, sample_rate: u32) -> Self {
        let gain_db = gain_db.clamp(-6.0, 6.0);
        let a = 10f32.powf(gain_db / 40.0);
        let w0 = 2.0 * std::f32::consts::PI * freq_hz / sample_rate as f32;
        let alpha = w0.sin() / (2.0 * q);

        let b0 = 1.0 + alpha * a;
        let b1 = -2.0 * w0.cos();
        let b2 = 1.0 - alpha * a;
        let a0 = 1.0 + alpha / a;
        let a1 = -2.0 * w0.cos();
        let a2 = 1.0 - alpha / a;

        Self {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }

    /// Process one sample through the biquad.
    pub fn process_sample(&mut self, x: f32) -> f32 {
        let y = self.b0 * x + self.b1 * self.x1 + self.b2 * self.x2
            - self.a1 * self.y1 - self.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = x;
        self.y2 = self.y1;
        self.y1 = y;
        y
    }

    /// Process a buffer in-place.
    pub fn process(&mut self, buffer: &mut [f32]) {
        for sample in buffer.iter_mut() {
            *sample = self.process_sample(*sample);
        }
    }
}

/// Apply a chain of parametric EQ bands to a buffer.
/// `bands` is a slice of `(freq_hz, gain_db, q)` tuples.
pub fn apply_eq(buffer: &mut [f32], bands: &[(f32, f32, f32)], sample_rate: u32) {
    for &(freq, gain, q) in bands {
        if gain.abs() < 0.01 {
            continue;
        } // skip flat bands
        let mut eq = BiquadEq::peaking(freq, gain, q, sample_rate);
        eq.process(buffer);
    }
}

/// Convert mono buffer to stereo with instrument-specific panning and width.
///
/// - `pan`: -1.0 = full left, 0.0 = center, +1.0 = full right
/// - `width`: 0.0 = mono (no Haas effect), 1.0 = full width (current default)
///
/// Uses constant-power panning: left = cos(θ), right = sin(θ)
/// where θ = (pan + 1) * π/4 (maps -1..+1 to 0..π/2).
pub fn stereo_pan_mix(mono: &[f32], sample_rate: u32, pan: f32, width: f32) -> Vec<f32> {
    let pan = pan.clamp(-1.0, 1.0);
    let width = width.clamp(0.0, 1.0);

    // Constant-power pan law
    let theta = (pan + 1.0) * std::f32::consts::FRAC_PI_4;
    let pan_left = theta.cos();
    let pan_right = theta.sin();

    // Haas delay for stereo width (same as existing mono_to_stereo but scaled by width)
    let delay_samples = ((0.0006 * width as f64) * sample_rate as f64) as usize;
    let len = mono.len();
    let mut stereo = vec![0.0f32; len * 2];

    let cross = 0.15 * width; // cross-feed scales with width

    for i in 0..len {
        let left_raw = mono[i];
        let right_raw = if delay_samples > 0 && i >= delay_samples {
            mono[i - delay_samples]
        } else if delay_samples == 0 {
            mono[i]
        } else {
            0.0
        };

        // Apply cross-feed then panning
        let left = (left_raw * (1.0 - cross) + right_raw * cross) * pan_left;
        let right = (right_raw * (1.0 - cross) + left_raw * cross) * pan_right;

        stereo[i * 2] = left;
        stereo[i * 2 + 1] = right;
    }

    stereo
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reverb_adds_tail() {
        let mut reverb = Reverb::hall(44100);
        // Create an impulse followed by silence
        let mut buffer = vec![0.0f32; 44100];
        buffer[0] = 1.0;

        reverb.process(&mut buffer);

        // The reverb tail should have energy well after the impulse
        let late_energy: f32 = buffer[4410..8820].iter().map(|s| s * s).sum();
        assert!(late_energy > 0.0001, "reverb should produce a tail");
    }

    #[test]
    fn test_reverb_preserves_dry() {
        let mut reverb = Reverb::new(44100, 0.0, 0.5, 0.5); // 100% dry
        let mut buffer = vec![0.5; 100];
        reverb.process(&mut buffer);
        for &s in &buffer {
            assert!((s - 0.5).abs() < 0.01, "dry signal should be preserved");
        }
    }

    #[test]
    fn test_mono_to_stereo_doubles_length() {
        let mono = vec![0.5; 100];
        let stereo = mono_to_stereo(&mono, 44100);
        assert_eq!(stereo.len(), 200);
    }

    #[test]
    fn test_mono_to_stereo_left_right_differ() {
        let mono: Vec<f32> = (0..1000).map(|i| (i as f32 * 0.01).sin()).collect();
        let stereo = mono_to_stereo(&mono, 44100);
        // Due to Haas delay, L and R shouldn't be identical
        let mut differ_count = 0;
        for i in 0..1000 {
            if (stereo[i * 2] - stereo[i * 2 + 1]).abs() > 0.001 {
                differ_count += 1;
            }
        }
        assert!(differ_count > 100, "L and R should differ for width");
    }

    #[test]
    fn test_soft_saturate_bounds() {
        let mut buffer = vec![-2.0, -1.0, 0.0, 1.0, 2.0];
        soft_saturate(&mut buffer, 2.0);
        for &s in &buffer {
            assert!(s >= -1.0 && s <= 1.0, "saturated signal should be in [-1, 1], got {}", s);
        }
    }

    #[test]
    fn test_soft_saturate_compresses() {
        let mut buffer = vec![0.5, -0.5, 0.8];
        soft_saturate(&mut buffer, 2.0);
        // With drive=2.0, input 0.5 -> tanh(1.0) ≈ 0.76, which is less than 1.0
        for &s in &buffer {
            assert!(s.abs() < 1.0, "saturated signals should be bounded");
        }
    }

    #[test]
    fn test_dattorro_reverb_adds_tail() {
        let mut reverb = DattorroReverb::hall(44100);
        let mut buffer = vec![0.0f32; 44100];
        buffer[0] = 1.0;
        reverb.process(&mut buffer);
        let late_energy: f32 = buffer[4410..8820].iter().map(|s| s * s).sum();
        assert!(late_energy > 0.0001, "dattorro should produce a tail");
    }

    #[test]
    fn test_dattorro_does_not_explode() {
        let mut reverb = DattorroReverb::new(44100, 0.5, 0.95, 0.2);
        let mut buffer = vec![0.0f32; 88200]; // 2 seconds
        buffer[0] = 1.0;
        reverb.process(&mut buffer);
        let max = buffer.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
        assert!(max < 10.0, "reverb should not explode, max={}", max);
    }

    #[test]
    fn test_biquad_eq_boost() {
        // Boosting should increase energy
        let sample_rate = 44100;
        let freq = 1000.0;
        let mut boosted = Vec::new();
        let mut flat = Vec::new();
        for i in 0..4410 {
            let t = i as f32 / sample_rate as f32;
            let s = (2.0 * std::f32::consts::PI * freq * t).sin();
            boosted.push(s);
            flat.push(s);
        }
        let mut eq = BiquadEq::peaking(1000.0, 6.0, 1.0, sample_rate);
        eq.process(&mut boosted);
        let flat_energy: f32 = flat.iter().map(|s| s * s).sum();
        let boost_energy: f32 = boosted.iter().map(|s| s * s).sum();
        assert!(
            boost_energy > flat_energy * 1.2,
            "EQ boost should increase energy"
        );
    }

    #[test]
    fn test_stereo_pan_left() {
        let mono = vec![1.0f32; 100];
        let stereo = stereo_pan_mix(&mono, 44100, -1.0, 0.0);
        // Full left pan: left channel should have signal, right should be near-zero
        let left_energy: f32 = stereo.iter().step_by(2).map(|s| s * s).sum();
        let right_energy: f32 = stereo.iter().skip(1).step_by(2).map(|s| s * s).sum();
        assert!(
            left_energy > right_energy * 10.0,
            "left pan should favor left channel"
        );
    }

    #[test]
    fn test_stereo_pan_center() {
        let mono = vec![1.0f32; 100];
        let stereo = stereo_pan_mix(&mono, 44100, 0.0, 0.0);
        // Center pan with no width: both channels should be equal
        let left_energy: f32 = stereo.iter().step_by(2).map(|s| s * s).sum();
        let right_energy: f32 = stereo.iter().skip(1).step_by(2).map(|s| s * s).sum();
        let ratio = (left_energy / right_energy).abs();
        assert!(
            (ratio - 1.0).abs() < 0.1,
            "center pan should be equal, ratio={}",
            ratio
        );
    }
}
