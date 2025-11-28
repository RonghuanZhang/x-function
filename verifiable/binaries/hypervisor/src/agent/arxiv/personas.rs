use anyhow::Result;
use rig::{
    client::CompletionClient,
    completion::{Prompt, PromptError},
    providers::anthropic::{self, completion::CompletionModel},
};

use crate::agent::arxiv::tool;

#[derive(Clone)]
pub struct Personas {
    inner: rig::agent::Agent<CompletionModel>,
}

impl Personas {
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("MINIMAX_API_KEY")?;

        let anthropic_client = anthropic::Client::builder(&api_key)
            .base_url("https://api.minimaxi.com/anthropic")
            .build()?;

        let agent = anthropic_client
            .agent("MiniMax-M2")
            .max_tokens(1000)
            .preamble(Self::system_prompt())
            .tool(tool::ArxivSearchTool)
            .build();

        Ok(Personas { inner: agent })
    }

    pub async fn process_search(&self, search: String) -> Result<String, PromptError> {
        // cap tool calls to 10 max
        self.inner.prompt(search).multi_turn(10).await
    }

    fn system_prompt() -> &'static str {
        r#"You are a helpful research assistant that can search and analyze academic papers from arXiv. \
        When asked about a research topic, use the search_arxiv tool to find relevant papers and \
        return only the raw JSON response from the tool."#
    }
}
