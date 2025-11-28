use crate::types::HypervisorState;
use axum::{extract::State, response::Json, routing::post, Router};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct SearchRequest {
    description: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SearchResponse {
    message: String,
}

pub fn api_register(router: Router<HypervisorState>) -> Router<HypervisorState> {
    router.route("/search", post(search_handler))
}

async fn search_handler(
    State(_state): State<HypervisorState>,
    Json(request): Json<SearchRequest>,
) -> Json<SearchResponse> {
    if request.description.to_lowercase().contains("arxiv") {
        Json(SearchResponse {
            message: "arxiv".to_string(),
        })
    } else {
        Json(SearchResponse {
            message: "no agent found".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::api::RouterRegister;

    use super::*;

    #[tokio::test]
    async fn test_search_with_arxiv() {
        let server = axum_test::TestServer::new(
            Router::new()
                .register_api(api_register)
                .with_state(HypervisorState::default()),
        )
        .unwrap();

        let response = server
            .post("/search")
            .json(&SearchRequest {
                description: "arxiv search".to_string(),
            })
            .await;

        response.assert_status_ok();
        let resp = response.json::<SearchResponse>();
        assert_eq!(resp.message, "arxiv agent here");
    }

    #[tokio::test]
    async fn test_search_without_arxiv() {
        let server = axum_test::TestServer::new(
            Router::new()
                .register_api(api_register)
                .with_state(HypervisorState::default()),
        )
        .unwrap();

        let response = server
            .post("/search")
            .json(&SearchRequest {
                description: "other search".to_string(),
            })
            .await;

        response.assert_status_ok();
        let resp = response.json::<SearchResponse>();
        assert_eq!(resp.message, "no agent found");
    }
}
