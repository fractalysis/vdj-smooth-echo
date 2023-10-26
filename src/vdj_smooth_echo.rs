extern crate baseplug;
extern crate serde;

use serde::{Deserialize, Serialize};
use baseplug::{Plugin, ProcessContext};
use fundsp::delay;
use typenum::U1;



baseplug::model! {
    #[derive(Debug, Serialize, Deserialize)]
    struct EchoModel {
        #[model(min = 0.001, max = 2.0)]
        #[parameter(name = "time", unit = "Generic",
            gradient = "Power(1.0)")]
        time: f32,

        #[model(min = 0.0, max = 1.0)]
        #[parameter(name = "time", unit = "Percentage")]
        feedback: f32
    }
}

impl Default for EchoModel {
    fn default() -> Self {
        Self {
            time: 0.5,
            feedback: 0.8
        }
    }
}


struct EchoPlug {
    delay_l: delay::Tap<U1,f32>,
    delay_r: delay::Tap<U1,f32>
}

impl Plugin for EchoPlug {
    const NAME: &'static str = "Smooth Echo";
    const PRODUCT: &'static str = "Smooth Echo";
    const VENDOR: &'static str = "Fractalysoft";

    const INPUT_CHANNELS: usize = 2;
    const OUTPUT_CHANNELS: usize = 2;

    type Model = EchoModel;

    #[inline]
    fn new(_sample_rate: f32, _model: &EchoModel) -> Self {
        EchoPlug {
            delay_l: delay::Tap::new(0.001, 2.0),
            delay_r: delay::Tap::new(0.001, 2.0)
        }
    }

    #[inline]
    fn process(&mut self, model: &EchoModelProcess, ctx: &mut ProcessContext<Self>) {
        let input = &ctx.inputs[0].buffers;
        let output = &mut ctx.outputs[0].buffers;

        for i in 0..ctx.nframes {
            output[0][i] = input[0][i];
            output[1][i] = input[1][i];
        }
        
    }
}


baseplug::vst2!(EchoPlug, b"FRse");