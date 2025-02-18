use bimap::BiHashMap;
use minidsp::formats::xml_config::Setting;
use strong_xml::XmlRead;

use super::spec::*;

pub struct Target {}
impl crate::Target for Target {
    fn filename() -> &'static str {
        "flexhtx.rs"
    }

    fn symbols() -> bimap::BiMap<String, usize> {
        symbols()
    }

    fn device() -> Device {
        device()
    }
}

pub(crate) fn input(input: usize) -> Input {
    Input {
        gate: Some(Gate {
            enable: format!("BM_DGain_{}_status", input + 1),
            gain: Some(format!("BM_DGain_{}", input + 1)),
        }),
        meter: Some(format!("Meter_In_{}", input + 1)),
        //meter_d: Some(format!("Meter_D_In_{}", input + 1)),
        peq: vec![], //no input peq
        /*bass_management: Some(Gate {
            enable: format!("BM_DGain_{}_status", 3 + output),
            gain: Some(format!("BM_DGain_{}", 3 + output)),
        }),*/
        routing: (0..8usize)
            .map(|output| Gate {
                enable: format!("BM_Mixer_{}_{}_status", input + 1, output + 1),
                gain: Some(format!("BM_Mixer_{}_{}", input + 1, output + 1)),
                //polarity: Some(format!("BM_Mixer_{input}_{output}_pol")),
            })
            .collect(),
    }
}

pub(crate) fn output(output: usize) -> Output {
    Output {
        gate: Gate {
            enable: format!("DGain_{}_0_status", output + 1),
            gain: Some(format!("DGain_{}_0", output + 1)),
        },
        /*routing: (0..7usize) //input number
        .map(|input| Gate {
            enable: format!("Out_Mixer_{input}_{output}_status"),
            gain: Some(format!("Out_Mixer_{input}_{output}")),
            polarity: Some(format!("Out_Mixer_{input}_{output}_pol")),
        })
        .collect(),*/
        meter: Some(format!("Meter_Out_{}", output + 1)), /*  (0..2usize)
                                                          .map(|ind| Some(format!("Meter_{}_{}", 1 + output, ind)))
                                                          .collect(),*/
        //meter_d: Some(format!("Meter_D_Out_{}", output + 1)),
        delay_addr: Some(format!("Delay_{}_0", 1 + output)),
        invert_addr: format!("polarity_out_{}_0", 1 + output),
        peq: (1..=10usize)
            .rev()
            .map(|index| format!("PEQ_{}_{}", output + 1, index))
            .collect(),
        xover: Some(Crossover {
            peqs: [1, 5]
                .iter()
                .map(|group| format!("BPF_{}_{}", output + 1, group))
                .chain(
                    [1, 5]
                        .iter()
                        .map(|group| format!("BM_BPF_{}_{}", output + 1, group)),
                )
                .collect(),
        }),
        compressor: Some(Compressor {
            bypass: format!("COMP_{}_0_status", output + 1),
            threshold: format!("COMP_{}_0_threshold", output + 1),
            ratio: format!("COMP_{}_0_ratio", output + 1),
            //knee: Some(format!("COMP_{}_0_knee", output + 1)),
            attack: format!("COMP_{}_0_atime", output + 1),
            release: format!("COMP_{}_0_rtime", output + 1),
            meter: Some(format!("Meter_Comp_{}", output + 1)),
        }),
        fir: None,
    }
}

pub fn device() -> Device {
    Device {
        product_name: "FlexHtx".into(),
        sources: vec![
            "Analog".into(),
            "Toslink".into(),
            "Spdif".into(),
            "Usb".into(),
            "Hdmi".into(),
        ],
        inputs: (0..8).map(input).collect(),
        outputs: (0..8).map(output).collect(),
        fir_max_taps: 0,
        internal_sampling_rate: 48000,
        ..Default::default()
    }
}

pub fn symbols() -> BiHashMap<String, usize> {
    let cfg = include_str!("config.xml");
    Setting::from_str(cfg).unwrap().name_map()
}

#[cfg(test)]
#[test]
fn test_codegen() {
    let mut symbol_map = symbols();
    let spec = device();
    super::generate_static_config(&mut symbol_map, &spec).to_string();
}
