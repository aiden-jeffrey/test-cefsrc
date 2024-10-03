use color_eyre::eyre::{eyre, Report, Result};
use gst::prelude::*;
use std::{
    sync::{Arc, RwLock},
    time::Duration,
};
use tokio::time::sleep;

use color_eyre::eyre::Error;

struct PipelineWrapper {
    pipeline: Arc<RwLock<gst::Element>>,
}

impl PipelineWrapper {
    pub fn new() -> Result<Self, Error> {
        let pipeline = gst::parse_launch(
            "cefsrc url=http://127.0.0.1:9000/ready_test.html
        sandbox=false do-timestamp=1
        chrome-extra-flags='no-sandbox disable-component-update' !
        video/x-raw,width=1280,height=720 ! videoconvert !
        x264enc threads=0 bitrate=5000 tune=zerolatency key-int-max=30 !
        mp4mux !
        filesink location=out.mp4",
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

    pub fn stop(&self) -> Result<(), Report> {
        match self.pipeline.write() {
            Ok(pipeline) => {
                pipeline.send_event(gst::event::Eos::new());
                pipeline.set_state(gst::State::Null)?;
                Ok(())
            }
            Err(_) => Err(eyre!("pause rc access")),
        }
    }
}

#[tokio::main]
async fn _main() -> Result<(), Error> {
    gst::init()?;

    let pipeline1 = Arc::new(RwLock::new(PipelineWrapper::new()?));

    pipeline1.write().unwrap().play()?;

    tokio::spawn(async move {
        println!("going to sleep");
        sleep(Duration::from_millis(5000)).await;
        pipeline1.read().unwrap().stop().unwrap();
        sleep(Duration::from_millis(5000)).await;
    })
    .await?;

    let pipeline2 = Arc::new(RwLock::new(PipelineWrapper::new()?));
    pipeline2.write().unwrap().play()?;

    tokio::spawn(async move {
        sleep(Duration::from_millis(5000)).await;
        pipeline2.read().unwrap().stop().unwrap();
    })
    .await?;

    Ok(())
}

pub fn main() {
    let _ = self::_main();
}
