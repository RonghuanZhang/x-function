use a2a_client::WebA2AClient;
use a2a_rs::{services::AsyncA2AClient, Part, TaskState};
use anyhow::Result;
use clap::{Parser, Subcommand};
use serde_json::json;
use uuid::Uuid;

#[derive(Parser)]
#[command(name = "cli")]
#[command(about = "CLI for hypervisor operations", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Search(Search),
    Deploy(Deploy),
    Call(Call),
}

#[derive(Parser)]
struct Search {
    #[arg(short, long)]
    server: String,

    description: String,
}

#[derive(Parser)]
struct Deploy {
    #[arg(short, long)]
    server: String,

    agent: String,
}

#[derive(Parser)]
struct Call {
    #[command(subcommand)]
    subcommand: CallSubcommands,
}

#[derive(Subcommand)]
enum CallSubcommands {
    Send(Send),
    GetSkills(GetSkills),
    GetAgentCard(GetAgentCard),
}

#[derive(Parser)]
struct Send {
    #[arg(short, long)]
    server: String,

    prompt: String,
}

#[derive(Parser)]
struct GetSkills {
    #[arg(short, long)]
    server: String,
}

#[derive(Parser)]
struct GetAgentCard {
    #[arg(short, long)]
    server: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Search(search) => {
            search_execute(&search.server, &search.description).await?;
        }
        Commands::Deploy(deploy) => {
            deploy_execute(&deploy.server, &deploy.agent).await?;
        }
        Commands::Call(call) => match call.subcommand {
            CallSubcommands::Send(send) => {
                send_execute(&send.server, &send.prompt).await?;
            }
            CallSubcommands::GetSkills(get_skills) => {
                get_skills_execute(&get_skills.server).await?;
            }
            CallSubcommands::GetAgentCard(get_card) => {
                get_agent_card_execute(&get_card.server).await?;
            }
        },
    }

    Ok(())
}

async fn search_execute(server: &str, description: &str) -> Result<()> {
    let client = reqwest::Client::new();

    let payload = json!({
        "description": description
    });

    let url = format!("{}/search", server);
    let response = client.post(url).json(&payload).send().await?;

    let response_text = response.text().await?;
    println!("Response: {}", response_text);

    Ok(())
}

async fn deploy_execute(server: &str, agent: &str) -> Result<()> {
    let client = reqwest::Client::new();

    let payload = json!({
        "agent": agent
    });

    let url = format!("{}/agent/deploy", server);
    let response = client.post(url).json(&payload).send().await?;

    let response_text = response.text().await?;
    println!("Response: {}", response_text);

    Ok(())
}

async fn send_execute(server: &str, prompt: &str) -> Result<()> {
    let client = WebA2AClient::auto_connect(server).await.unwrap();

    let message = a2a_rs::Message::builder()
        .role(a2a_rs::Role::User)
        .parts(vec![Part::Text {
            text: prompt.to_string(),
            metadata: None,
        }])
        .message_id(Uuid::now_v7().to_string())
        .build();

    let task_id = Uuid::now_v7().to_string();
    let _task = client
        .http
        .send_task_message(&task_id, &message, None, None)
        .await
        .unwrap();

    let completed_task;

    loop {
        let task = client.http.get_task(&task_id, None).await.unwrap();

        match task.status.state {
            TaskState::Submitted => (),
            TaskState::Working => (),
            TaskState::Completed => {
                completed_task = task;
                break;
            }
            _ => unreachable!("unexpected task status {:?}", task.status),
        }

        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }

    println!("Response: {:?}", completed_task);

    Ok(())
}

async fn get_skills_execute(server: &str) -> Result<()> {
    let client = reqwest::Client::new();

    let url = format!("{}/skills", server);
    let response = client.get(url).send().await?;

    let response_text = response.text().await?;
    println!("Response: {}", response_text);

    Ok(())
}

async fn get_agent_card_execute(server: &str) -> Result<()> {
    let client = reqwest::Client::new();

    let url = format!("{}/agent-card", server);
    let response = client.get(url).send().await?;

    let response_text = response.text().await?;
    println!("Response: {}", response_text);

    Ok(())
}
