use a2a_rs::{DefaultRequestProcessor, HttpServer, InMemoryTaskStorage};
use anyhow::Result;

use crate::agent::a2a::protocol::A2AProtocol;

pub struct A2AServer {}

// TODO: make it to riscv guest, interactive throught io
impl A2AServer {
    pub async fn start(agent: impl A2AProtocol, addr: impl AsRef<str>) -> Result<()> {
        let agent_info = agent.agent_info();
        let storage = InMemoryTaskStorage::default();
        let protocol_handler = agent.protocol_message_handler(storage.clone());

        let processor = DefaultRequestProcessor::new(
            protocol_handler,
            storage.clone(),
            storage,
            agent_info.clone(),
        );

        let server = HttpServer::new(processor, agent_info, addr.as_ref().to_string());

        server.start().await?;

        Ok(())
    }
}
