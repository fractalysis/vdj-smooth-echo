extern crate baseplug;
extern crate serde;

use serde::{Deserialize, Serialize};
use baseplug::{Plugin, ProcessContext};
use fundsp::{delay, audionode::Frame, prelude::AudioNode, signal::new_signal_frame};
use typenum::U1;



baseplug::model! {
    #[derive(Debug, Serialize, Deserialize)]
    struct EchoModel {
        #[model(min = 0.001, max = 2.0)]
        #[parameter(name = "time", unit = "Generic",
            gradient = "Power(1.0)")]
        time: f32,

        #[model(min = 0.0, max = 1.0)]
        #[parameter(name = "mix", unit = "Percentage")]
        mix: f32,

        #[model(min = 0.0, max = 1.0)]
        #[parameter(name = "time", unit = "Percentage")]
        feedback: f32,
    }
}

impl Default for EchoModel {
    fn default() -> Self {
        Self {
            time: 0.5,
            mix: 0.0,
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
        let mut ret = EchoPlug {
            delay_l: delay::Tap::new(0.001, 2.0),
            delay_r: delay::Tap::new(0.001, 2.0)
        };

        ret.delay_l.set_sample_rate(_sample_rate as f64);
        ret.delay_r.set_sample_rate(_sample_rate as f64);

        ret
    }

    #[inline]
    fn process(&mut self, model: &EchoModelProcess, ctx: &mut ProcessContext<Self>) {
        let input = &ctx.inputs[0].buffers;
        let output = &mut ctx.outputs[0].buffers;

        for i in 0..ctx.nframes {
            let delayed_frame_l = self.delay_l.tick(&Frame::from([
                input[0][i], model.time[i]
            ]));
            let delayed_frame_r = self.delay_r.tick(&Frame::from([
                input[1][i], model.time[i]
            ]));

            output[0][i] = input[0][i] * (1. - model.mix[i]) + delayed_frame_l[0] * model.mix[i];
            output[1][i] = input[1][i] * (1. - model.mix[i]) + delayed_frame_r[0] * model.mix[i];
        }
        
    }
}


baseplug::vst2!(EchoPlug, b"FRse");