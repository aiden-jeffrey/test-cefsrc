use color_eyre::eyre::{eyre, Report, Result};
use gst::prelude::*;
use std::sync::{Arc, RwLock};

use color_eyre::eyre::Error;

#[derive(PartialEq)]
pub enum BusCommandType {
    Eos,
}

struct PipelineWrapper {
    pipeline: Arc<RwLock<gst::Element>>,
}

impl PipelineWrapper {
    pub fn new(filename: &str) -> Result<Self, Error> {
        let pipeline = gst::parse_launch(
            &format!(
                "cefsrc url=http://localhost:9000/ready_test.html
                 listen-for-js-signals=true sandbox=false do-timestamp=1
                 chrome-extra-flags='no-sandbox disable-component-update' !
                 video/x-raw,width=1280,height=720 ! videoconvert !
                 x264enc threads=0 bitrate=5000 tune=zerolatency key-int-max=30 !
                 mp4mux !
                 filesink location={}.mp4"
            , filename),
        )?;

        let pipeline_wrapper = PipelineWrapper {
            pipeline: Arc::new(RwLock::new(pipeline)),
        };

        Ok(pipeline_wrapper)
    }

    pub fn play(&self) -> Result<(), Report> {
        match self.pipeline.write() {
            Ok(pipeline) => {
                pipeline.set_state(gst::State::Playing)?;
                Ok(())
            }
            Err(_) => Err(eyre!("play rc access")),
        }
    }

    pub fn handle_pipeline_message(&self) -> Option<BusCommandType> {
        use gst::message::MessageView;

        if let Ok(pipeline) = self.pipeline.read() {
            if let Some(Some(message)) = pipeline.bus().map(|bus| bus.pop()) {
                if let MessageView::Eos(_) = message.view() {
                    if pipeline.set_state(gst::State::Null).is_ok() {
                        return Some(BusCommandType::Eos);
                    }
                }
            }
        }

        None
    }
}

#[tokio::main]
async fn _main() -> Result<(), Error> {
    gst::init()?;

    let pipeline1 = Arc::new(RwLock::new(PipelineWrapper::new("1")?));

    pipeline1.write().unwrap().play()?;

    tokio::spawn(async move {
        loop {
            let mut saw_eos = false;
            while let Some(cmd) = pipeline1.read().unwrap().handle_pipeline_message() {
                if cmd == BusCommandType::Eos {
                    saw_eos = true;
                    break;
                }
            }

            if saw_eos {
                break;
            }
        }
    })
    .await?;

    let pipeline2 = Arc::new(RwLock::new(PipelineWrapper::new("2")?));
    pipeline2.write().unwrap().play()?;

    tokio::spawn(async move {
        loop {
            let mut saw_eos = false;
            while let Some(cmd) = pipeline2.read().unwrap().handle_pipeline_message() {
                if cmd == BusCommandType::Eos {
                    saw_eos = true;
                    break;
                }
            }

            if saw_eos {
                break;
            }
        }
    })
    .await?;

    Ok(())
}

pub fn main() {
    let _ = self::_main();
}