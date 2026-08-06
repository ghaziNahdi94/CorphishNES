const CPU_CLOCK_FREQUENCY: f32 = 1789773.0;
const DUTY_CYCLE_TABLE: [[u8; 8]; 4] = [
    [0, 1, 0, 0, 0, 0, 0, 0], // Duty 0 : 12.5%
    [0, 1, 1, 0, 0, 0, 0, 0], // Duty 1 : 25%   
    [0, 1, 1, 1, 1, 0, 0, 0], // Duty 2 : 50%   
    [1, 0, 0, 1, 1, 1, 1, 1], // Duty 3 : 75%   (Reverse Duty 1)
];

const NES_LENGTH_COUNTER_TABLE:[u8; 32] = [10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14, 12, 16, 24, 18, 48, 20, 96, 22, 192, 24, 72, 26, 16, 28, 32, 30];

const MODE_4_STEP_THRESHOLDS: [u16; 4] = [7457, 14913, 22371, 29829];

struct PulseChannel {
    // bit(7) : Duty 1; bit(6): Duty 2; bit(5): Len Halt
    // bit(4): Const Volume; bit(0-3): Volume
    //------------------------------------
    //"bit(6-7)" => Duty (Wave format [0-3])
    //"Len Halt" => 1: Sound will never stop (no length counter)
    //"Const Volume" => 1: Fixed volume; 0: Activated envelope
    //"Volume" => 0b1111 = 15 maximum
    control_register: u8, 

    sweep_register: u8,
    time_low_register: u8,
    time_high_register: u8,

    //Internal state
    enabled: bool,

    duty_step_counter: u8,   //Step counter (0-7) for the 8 steps ; Increment evrey time the Timer become 0
    length_counter: u8,
    timer: u16,

    envelope_generated_volume: u8,
    envelope_counter: u8,
    envelope_period: u8,
    envelope_start: bool,
}

impl PulseChannel {
    fn new() -> PulseChannel {
        PulseChannel {
            control_register: 0u8,
            sweep_register: 0u8,
            time_low_register: 0u8,
            time_high_register: 0u8,
            duty_step_counter: 0u8,
            length_counter: 0u8,
            envelope_generated_volume: 0u8,
            envelope_counter: 0u8,
            envelope_period: 0u8,
            envelope_start: false,
            enabled: true,
            timer: 0u16,
        }
    }

    pub fn step(&mut self) {
        if self.timer == 0 {
            self.timer = self.get_timer_period();
            self.duty_step_counter = (self.duty_step_counter + 1) & 7;
        } else {
            self.timer -= 1;
        }

    }

    pub fn write_to_control_register(&mut self, value: u8) {
        self.control_register = value;
        self.envelope_period = value & 0x0F;
    }

    pub fn write_to_time_low_register(&mut self, value: u8) {
        self.time_low_register = value;
    }

    pub fn write_to_time_high_register(&mut self, value: u8) {
        self.time_high_register = value & 0b00000111;

        let lookup_table_index = (value >> 3) & 0b00011111;
        self.length_counter = NES_LENGTH_COUNTER_TABLE[lookup_table_index as usize];

        self.start_envelope();
    }

    pub fn output(&self) -> u8 {
        if self.enabled == false {
            return 0;
        }

        if self.length_counter == 0 {
            return 0;
        }

        if self.get_timer_period() < 8 {
            return 0;
        } 

        let duty_cycle = self.get_duty_cycle();
        let wave_value = duty_cycle[self.duty_step_counter as usize];
        if wave_value == 0 {
            return 0;
        }

        return self.get_volume_to_use();
    }

    fn start_envelope(&mut self) {
        self.envelope_start = true;
        self.envelope_generated_volume = 15;

        self.envelope_counter = self.envelope_period;
    }

    pub fn clock_length_counter(&mut self) {
        if self.length_counter > 0 && self.get_length_halt() == 0 {
            self.length_counter -= 1;
        }
    }

    pub fn clock_envelope(&mut self) {
        if self.envelope_start {
            self.envelope_start = false;
            self.envelope_generated_volume = 15;
            self.envelope_counter = self.envelope_period;
        } else {
            if self.envelope_counter > 0 {
                self.envelope_counter -= 1;
            } else {
                self.envelope_counter = self.envelope_period;
                
                let envelope_loop = self.get_length_halt() == 1; 
                
                if envelope_loop {
                    self.envelope_generated_volume = 15;
                } else if self.envelope_generated_volume > 0 {
                    self.envelope_generated_volume -= 1;
                }
            }
        }
    }

    fn get_volume_to_use(&self) -> u8 {
        if self.get_const_volume() == 0 {
            self.envelope_generated_volume
        } else {
            self.control_register & 0b00001111
        }
    }

    fn get_duty_cycle(&self) -> [u8; 8] {
        let index = (self.control_register >> 6) & 0b0011;
        DUTY_CYCLE_TABLE[index as usize]
    }

    fn get_length_halt(&self) -> u8 {
        (self.control_register >> 5) & 0b0001
    }

    fn get_const_volume(&self) -> u8 {
        (self.control_register >> 4) & 0b0001
    }

    fn get_volume(&self) -> u8 {
        self.control_register & 0b1111
    }

    fn get_timer_period(&self) -> u16 {
        (self.time_high_register as u16) << 8 | (self.time_low_register as u16)
    }

    fn calculate_frequency(&self) -> f32 {
        let timer = self.get_timer_period();
        CPU_CLOCK_FREQUENCY / ((16 * ((timer) + 1)) as f32)
    }
}

struct Triangle;

struct Noise;

struct Dmc;

#[derive(Clone, Copy, PartialEq)]
enum StepMode {
    FourSteps,
    FiveSteps
}

struct FrameCounter {
    cpu_cycle_counter: u16, //count CPU cycles
    step: u8,
    irq_enabled: bool,
    mode: StepMode,
}

impl FrameCounter {
    fn new() -> Self {
        Self {
            cpu_cycle_counter: 0,
            step: 0,
            irq_enabled: false,
            mode: StepMode::FourSteps
        }
    }

    fn step(&mut self) -> bool {
        self.cpu_cycle_counter += 1;

        let threshold = MODE_4_STEP_THRESHOLDS[self.step as usize];

        if self.cpu_cycle_counter >= threshold {
            self.step+= 1;

            let max_step = match self.mode {
                StepMode::FourSteps => 3,
                StepMode::FiveSteps => 4,
            };

            if self.step == max_step {
                self.step = 0;
                self.cpu_cycle_counter = 0;
            }

            return true;
        }

        false
    }
}

pub struct Apu {
    pulse1: PulseChannel,
    pulse2: PulseChannel,
    triangle: Triangle,
    noise: Noise,
    dmc: Dmc,
    frame_counter: FrameCounter,

    sample_counter: f32,
    audio_buffer: Vec<f32>
}

impl Apu {
    pub fn new() -> Apu {
        Apu {
            pulse1: PulseChannel::new(),
            pulse2: PulseChannel::new(),
            frame_counter: FrameCounter::new(),
            triangle: Triangle,
            noise: Noise,
            dmc: Dmc,
            sample_counter: 0f32,
            audio_buffer: Vec::new()
            //TODO
        }
    }

    pub fn save_to_wav(&self, filename: &str) {
        use hound::{WavSpec, WavWriter, SampleFormat};
        
        let spec = WavSpec {
            channels: 1,
            sample_rate: 48000,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        };
        
        let mut writer = WavWriter::create(filename, spec)
            .expect("WAV file creation failed !");
        
        for &sample in &self.audio_buffer {
            let i16_sample = (sample * 32767.0) as i16;
            writer.write_sample(i16_sample).unwrap();
        }
        
        writer.finalize().unwrap();
    }

    pub fn step_and_collect_sound(&mut self) {
        self.step();

        self.sample_counter += 1.0;
        
        if self.sample_counter >= 37.3 {
            self.sample_counter -= 37.3;
            
            let sample = self.generate_sample();
            self.audio_buffer.push(sample);
        }
    }

    fn step(&mut self) {
        let frame_counter_ticked = self.frame_counter.step();
        if frame_counter_ticked {

            // --- PULSE 1 ---
            self.pulse1.clock_envelope();

            if self.frame_counter.step == 1 || self.frame_counter.step == 3 {
                self.pulse1.clock_length_counter();
            }

            // --- PULSE 2 ---
            self.pulse2.clock_envelope();

            if self.frame_counter.step == 1 || self.frame_counter.step == 3 {
                self.pulse2.clock_length_counter();
            }
        }

        self.pulse1.step();
        self.pulse2.step();
    }

    fn generate_sample(&self) -> f32 {
        let pulse1_out = self.pulse1.output() as f32;
        let pulse2_out = self.pulse2.output() as f32;
        
        // Normalize output between 0.0 et 1.0
        let mixed = (pulse1_out + pulse2_out) / 30.0; 

        
        // TODO Triangle, Noise, DMC using Blargg formula
        mixed.min(1.0) 
    }

    pub fn read_cpu(&self, cpu_address: u16) -> u8 {
        match cpu_address {
            0x4015 => {
                //Bits:
                // 0 => Length Counter of Pulse 1
                // 1 => Length Counter of Pulse 2
                // 2 => Length Counter of Triangle
                // 3 => Length Counter of Noise
                // 4 => DMC state
                // 5-7 => unused 0
                let mut status = 0;
                
                if self.pulse1.length_counter > 0 { status |= 0b00000001 }
                if self.pulse2.length_counter > 0 { status |= 0b00000010 }
                //TODO TRIANGLE DMC NOISE

                status
            },
            _ => 0
        }
    }

    pub fn write_cpu(&mut self, cpu_address: u16, value: u8) {
        match cpu_address {
            0x4000 => self.pulse1.write_to_control_register(value),
            0x4002 => self.pulse1.write_to_time_low_register(value),
            0x4003 => self.pulse1.write_to_time_high_register(value),
            0x4004 => self.pulse2.write_to_control_register(value),
            0x4006 => self.pulse2.write_to_time_low_register(value),
            0x4007 => self.pulse2.write_to_time_high_register(value),
            0x4015 => {
                self.pulse1.enabled = value & 0b00000001 != 0;
                self.pulse2.enabled = value & 0b00000010 != 0;
                //TODO TRIANGLE DMC NOISE
            }
            0x4017 => {
                let is_irq_enabled = (value >> 6) & 0b00001 != 0;
                self.frame_counter.irq_enabled = is_irq_enabled;

                let mode = if value >> 7 != 0 { StepMode::FiveSteps } else { StepMode::FourSteps };
                self.frame_counter.mode = mode;
            }
            _ => {}
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pulse_basic_beep() {
        let mut pulse = PulseChannel::new();
        
        // Volume constant = 15, Duty = 50%
        pulse.write_to_control_register(0x4F); 
        // Timer Low = 0x00
        pulse.write_to_time_low_register(0x00);
        // Timer High = 0x04, Length = max
        pulse.write_to_time_high_register(0x04); 
        
        pulse.enabled = true;

        // Simulate 1000 CPU cycles
        for _ in 0..1000 {
            pulse.step();
        }

        // We produce a sound !!
        assert!(pulse.output() > 0, "Pulse channel is able to produce sound");
    }
}