extern crate baseplug;
extern crate serde;

use serde::{Deserialize, Serialize};
use baseplug::{Plugin, ProcessContext};
use fundsp::{delay, filter, audionode::Frame, prelude::AudioNode};
use typenum::U1;


// Todo:
// - (Insane Mode) Smoothed snapping of the time parameter over a full half second to 1/2, 1/4, and 1/8 notes if you are within ~1 semitone
//     (e.g. at 120 bpm a half note is 0.5s, so anywhere from 0.4719s to 0.5297s should smoothly snap to 0.5s)
//     0.5s * 2^( +- 0.083 )


// Experimental way of turning down the volume as the feedback goes up
// This multiplies the input by (1-VCC*feedback) before putting into the chain,
//   so VCC=1 assumes the amplitude is proportional to a geometric series (1/(1-feedback))
// Set to 0 to not turn down the volume at all
const VOLUME_CONTROL_COEF: f32 = 0.92;

// The hipass applied after the delay since it can add low frequencies
const HIPASS_AT: f32 = 10.; // In Hz


baseplug::model! {
    #[derive(Debug, Serialize, Deserialize)]
    struct EchoModel {
        #[model(min = 0.001, max = 2.0, smooth_ms = 100)]
        #[parameter(name = "time", unit = "Generic", gradient = "Power(4.0)")]
        time: f32,

        #[model(min = 0.0, max = 1.0)]
        #[parameter(name = "feedback", unit = "Percentage")]
        feedback: f32,
    }
}

impl Default for EchoModel {
    fn default() -> Self {
        Self {
            time: 0.5,
            feedback: 0.0
        }
    }
}


struct EchoPlug {
    delay_l: delay::Tap<U1,f32>,
    delay_r: delay::Tap<U1,f32>,

    hipass_l: filter::Highpole<f32,f32,U1>,
    hipass_r: filter::Highpole<f32,f32,U1>,

    last_sample_l: f32,
    last_sample_r: f32
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
            delay_r: delay::Tap::new(0.001, 2.0),

            hipass_l: filter::Highpole::new(HIPASS_AT),
            hipass_r: filter::Highpole::new(HIPASS_AT),

            last_sample_l: 0.,
            last_sample_r: 0.
        };

        ret.delay_l.set_sample_rate(_sample_rate as f64);
        ret.delay_r.set_sample_rate(_sample_rate as f64);

        ret.hipass_l.set_sample_rate(_sample_rate as f64);
        ret.hipass_r.set_sample_rate(_sample_rate as f64);

        ret
    }

    #[inline]
    fn process(&mut self, model: &EchoModelProcess, ctx: &mut ProcessContext<Self>) {
        let input = &ctx.inputs[0].buffers;
        let output = &mut ctx.outputs[0].buffers;

        for i in 0..ctx.nframes {

            if model.feedback[i] > 0.99 { // Lock the samples into an infinite loop
                let delayed_sample_l = self.delay_l.tick(&Frame::from([
                    self.last_sample_l, model.time[i]
                ]))[0];
                let delayed_sample_r = self.delay_r.tick(&Frame::from([
                    self.last_sample_r, model.time[i]
                ]))[0];

                self.last_sample_l = delayed_sample_l;
                self.last_sample_r = delayed_sample_r;
            }

            else {
                let delayed_sample_l = self.delay_l.tick(&Frame::from([
                    input[0][i] * ( 1. - VOLUME_CONTROL_COEF * model.feedback[i] ) + self.last_sample_l * model.feedback[i], model.time[i]
                ]))[0];
                let delayed_sample_r = self.delay_r.tick(&Frame::from([
                    input[1][i] * ( 1. - VOLUME_CONTROL_COEF * model.feedback[i] ) + self.last_sample_r * model.feedback[i], model.time[i]
                ]))[0];

                self.last_sample_l = input[0][i] + delayed_sample_l * model.feedback[i];
                self.last_sample_r = input[1][i] + delayed_sample_r * model.feedback[i];
            }

            output[0][i] = self.hipass_l.tick(&Frame::from([self.last_sample_l]))[0];
            output[1][i] = self.hipass_r.tick(&Frame::from([self.last_sample_r]))[0];
        }
        
    }
}


baseplug::vst2!(EchoPlug, b"FRse");