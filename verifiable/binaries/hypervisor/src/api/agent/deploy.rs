use crate::agent::a2a::server::A2AServer;
use crate::agent::arxiv::personas::Personas;
use crate::error::HypervisorError;
use crate::types::HypervisorState;
use anyhow::{anyhow, Context};
use axum::http::StatusCode;
use axum::{extract::State, response::Json, routing::post, Router};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct DeployRequest {
    agent: String,
}

#[derive(Deserialize, Serialize)]
struct DeployResponse {
    message: String,
}

pub fn api_register(router: Router<HypervisorState>) -> Router<HypervisorState> {
    router.route("/agent/deploy", post(deploy_handler))
}

async fn deploy_handler(
    State(state): State<HypervisorState>,
    Json(request): Json<DeployRequest>,
) -> Result<Json<DeployResponse>, HypervisorError> {
    // Validate agent name - only "arxiv" is allowed for now
    if request.agent.to_lowercase() == "arxiv" {
        if state.get_running_agent_name().await.as_ref() == Some(&request.agent) {
            return Ok(Json(DeployResponse {
                message: format!("Agent '{}' already deployed", request.agent),
            }));
        }

        state.stop_running_agent().await;

        let agent = Personas::from_env()
            .context("start agent")
            .context(StatusCode::INTERNAL_SERVER_ERROR)
            .map_err(|e| anyhow!(e.to_string()))?;

        let handle = tokio::spawn(A2AServer::start(agent, "127.0.0.1:3000"));

        state.set_agent(request.agent.clone(), handle).await;

        Ok(Json(DeployResponse {
            message: format!("Agent '{}' deployed successfully", request.agent),
        }))
    } else {
        Err(HypervisorError::InvalidRequest(
            "unknown agent".to_string(),
            StatusCode::BAD_REQUEST,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_deploy_arxiv_agent() {
        use crate::api::RouterRegister;

        let state = HypervisorState::default();
        let server = axum_test::TestServer::new(
            Router::new()
                .register_api(api_register)
                .with_state(state.clone()),
        )
        .unwrap();

        let response = server
            .post("/agent/deploy")
            .json(&DeployRequest {
                agent: "arxiv".to_string(),
            })
            .await;

        response.assert_status_ok();
        let resp = response.json::<DeployResponse>();
        assert_eq!(resp.message, "Agent 'arxiv' deployed successfully");
    }

    #[tokio::test]
    async fn test_deploy_unknown_agent() {
        use crate::api::RouterRegister;

        let server = axum_test::TestServer::new(
            Router::new()
                .register_api(api_register)
                .with_state(HypervisorState::default()),
        )
        .unwrap();

        let response = server
            .post("/agent/deploy")
            .json(&DeployRequest {
                agent: "unknown_agent".to_string(),
            })
            .await;

        response.assert_status_bad_request();
    }
}
