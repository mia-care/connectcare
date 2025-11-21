pub mod event;
pub mod processors;
pub mod sinks;
pub mod executor;

use tokio::sync::mpsc;
use event::PipelineEvent;

pub type PipelineSender = mpsc::Sender<PipelineEvent>;
pub type PipelineReceiver = mpsc::Receiver<PipelineEvent>;

pub fn create_pipeline_channel(buffer_size: usize) -> (PipelineSender, PipelineReceiver) {
    mpsc::channel(buffer_size)
}
