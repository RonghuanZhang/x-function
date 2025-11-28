use a2a_rs::{
    A2AError, AsyncMessageHandler, AsyncTaskManager, InMemoryTaskStorage, Message, Part, Role,
    SimpleAgentInfo, Task, TaskState,
};
use async_trait::async_trait;
use uuid::Uuid;

use crate::agent::arxiv::personas::Personas;

#[async_trait]
pub trait A2AProtocol {
    fn agent_info(&self) -> SimpleAgentInfo;
    fn protocol_message_handler(&self, _: InMemoryTaskStorage) -> A2AProtocolHandler;
}

#[async_trait]
impl A2AProtocol for Personas {
    fn agent_info(&self) -> SimpleAgentInfo {
        SimpleAgentInfo::new(
            "arxiv agent".to_string(),
            "http://localhost:3000".to_string(),
        )
        .with_description("agent that helps you search paper on arxiv".to_string())
        .add_comprehensive_skill(
            "search_arxiv".to_string(),
            "Search Arxiv".to_string(),
            Some("Search paper on arxiv".to_string()),
            Some(vec!["arxiv".to_string(), "paper".to_string()]),
            Some(vec![
                "Please give me latest 5 papers about lattice zero knowledge".to_string(),
            ]),
            Some(vec!["text".to_string(), "data".to_string()]),
            Some(vec!["text".to_string(), "data".to_string()]),
        )
    }

    fn protocol_message_handler(&self, task_manager: InMemoryTaskStorage) -> A2AProtocolHandler {
        A2AProtocolHandler::new(self.clone(), task_manager)
    }
}

// FIX: hard code
#[derive(Clone)]
pub struct A2AProtocolHandler {
    agent: Personas,
    task_manager: InMemoryTaskStorage,
}

impl A2AProtocolHandler {
    pub(crate) fn new(agent: Personas, task_manager: InMemoryTaskStorage) -> Self {
        Self {
            agent,
            task_manager,
        }
    }
}

#[async_trait]
impl AsyncMessageHandler for A2AProtocolHandler {
    async fn process_message<'a>(
        &self,
        task_id: &'a str,
        message: &'a Message,
        _session_id: Option<&'a str>,
    ) -> Result<Task, A2AError> {
        if self.task_manager.task_exists(task_id).await? {
            return self.task_manager.get_task(task_id, None).await;
        }

        let context_id = { message.context_id.as_ref() }
            .map(|id| id.to_string())
            .unwrap_or_else(|| Uuid::now_v7().to_string());

        self.task_manager.create_task(task_id, &context_id).await?;

        let search_text = extract_search_text_from_message(message);

        let parts = match self.agent.process_search(search_text).await {
            Ok(result) => {
                vec![Part::text(result)]
            }
            Err(err) => {
                vec![Part::text(err.to_string())]
            }
        };

        let resp_msg = Message::builder()
            .role(Role::Agent)
            .parts(parts)
            .message_id(Uuid::now_v7().to_string())
            .build();

        self.task_manager
            .update_task_status(task_id, TaskState::Completed, Some(resp_msg))
            .await?;

        self.task_manager.get_task(task_id, None).await
    }
}

fn extract_search_text_from_message(message: &Message) -> String {
    let mut text_parts = vec![];
    for part in message.parts.iter() {
        if let Part::Text { text, .. } = part {
            text_parts.push(text.clone())
        }
    }

    text_parts.join(" ")
}
