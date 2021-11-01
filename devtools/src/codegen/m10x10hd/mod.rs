use bimap::BiHashMap;
use minidsp::{formats::xml_config::Setting, AddrEncoding, Dialect, FloatEncoding};
use strong_xml::XmlRead;

use super::spec::*;

pub struct Target {}
impl crate::Target for Target {
    fn filename() -> &'static str {
        "m10x10hd.rs"
    }

    fn symbols() -> bimap::BiMap<String, usize> {
        symbols()
    }

    fn device() -> Device {
        device()
    }
}

pub(crate) fn routing(
    input: usize,
    num_inputs: usize,
    num_outputs: usize,
    spacing: usize,
) -> Vec<Gate> {
    // Routing settings *BY OUTPUT CHANNEL*

    let syms = |start: u8| -> Vec<String> {
        (start..(start + num_inputs as u8))
            .map(|n| format!("MuteNoSlewAlg{}mute", n))
            .collect()
    };
    let starts: Vec<_> = (11..)
        .step_by(spacing)
        .filter(|x| *x != 71)
        .take(num_outputs)
        .collect();

    let outputs = starts.iter().map(|&x| syms(x));
    outputs
        .map(|inputs| Gate {
            enable: inputs[input].clone(),
            gain: None,
        })
        .collect()
}

pub(crate) fn input(
    input: usize,
    num_inputs: usize,
    num_outputs: usize,
    routing_spacing: usize,
) -> Input {
    Input {
        gate: Some(Gate {
            enable: format!("MuteNoSlewAlg{}mute", input + 71),
            gain: Some(format!("Gain1940AlgNS{}", input + 11)),
        }),
        meter: None,
        peq: (0..6usize)
            .map(|index| format!("PEQ_{}_{}", input + 11, 6 - index))
            .collect(),
        routing: routing(input, num_inputs, num_outputs, routing_spacing),
    }
}

pub(crate) fn output(output: usize) -> Output {
    Output {
        gate: Gate {
            enable: format!("MuteNoSlewAlg{}mute", output + 1),
            gain: Some(format!("Gain1940AlgNS{}", output + 1)),
        },
        meter: None,
        delay_addr: Some(format!("MultCtrlDelGrowAlg{}", output + 1)),
        invert_addr: format!("EQ1940Invert{}gain", output + 1),
        peq: (1..=5usize)
            .rev()
            .map(|index| format!("PEQ_{}_{}", output + 1, index))
            .collect(),
        xover: Some(Crossover {
            peqs: [1, 5]
                .iter()
                .map(|group| format!("BPF_{}_{}", output + 1, group))
                .collect(),
        }),
        // This device has output compressors, but uses specific commands that haven't been reversed yet
        compressor: None,
        fir: None,
    }
}

pub fn device() -> Device {
    #[allow(clippy::needless_update)]
    Device {
        product_name: "MiniDSP 10x10HD".into(),
        sources: vec!["Toslink".into(), "Spdif".into()],
        inputs: (0..10).map(|n| input(n, 10, 10, 10)).collect(),
        outputs: (0..10).map(output).collect(),
        fir_max_taps: 0,
        internal_sampling_rate: 96000,
        dialect: Dialect {
            addr_encoding: AddrEncoding::AddrLen2,
            float_encoding: FloatEncoding::FixedPoint,
        },
        ..Default::default()
    }
}

pub fn symbols() -> BiHashMap<String, usize> {
    let cfg = include_str!("config.xml");
    Setting::from_str(cfg).unwrap().name_map()
}
