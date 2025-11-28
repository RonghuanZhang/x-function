use std::sync::Arc;

use k256::{
    ecdsa::{SigningKey, VerifyingKey},
    EncodedPoint,
};
use tokio::{sync::Mutex, task::JoinHandle};
use uuid::Uuid;

use crate::Config;

#[derive(Clone, Default)]
pub struct HypervisorState {
    pub config: Config,
    session_key_pairs: SessionKeyPairs,
    agent_service: Arc<Mutex<Option<AgentService>>>,
}

impl HypervisorState {
    pub fn new(config: Config) -> Self {
        HypervisorState {
            config,
            ..Default::default()
        }
    }

    pub async fn set_agent(&self, agent_name: String, handle: JoinHandle<anyhow::Result<()>>) {
        let mut agent_service = self.agent_service.lock().await;

        if let Some(a) = agent_service.as_mut() {
            a.name = agent_name;
            a.service_handle = handle;
        }
    }

    pub async fn get_running_agent_name(&self) -> Option<String> {
        let agent_service = self.agent_service.lock().await;
        agent_service.as_ref().map(|a| a.name.clone())
    }

    pub async fn stop_running_agent(&self) {
        let mut agent_service = self.agent_service.lock().await;

        if let Some(a) = agent_service.as_mut() {
            a.service_handle.abort();
        }

        *agent_service = None;
    }

    #[cfg(test)]
    pub fn set_session_key_pairs(&mut self, session_key_pairs: SessionKeyPairs) {
        self.session_key_pairs = session_key_pairs;
    }

    pub fn create_session_keypair(self, pubkey: &VerifyingKey) -> (VerifyingKey, Uuid) {
        self.session_key_pairs.create(pubkey)
    }

    pub fn get_session_keypair(self, pubkey: &VerifyingKey) -> Option<(SigningKey, Uuid)> {
        self.session_key_pairs
            .0
            .get(&pubkey.to_encoded_point(true))
            .map(|i| i.to_owned())
    }
}

pub struct ServerContext {
    pub state: HypervisorState,
}

#[derive(Clone, Default)]
pub struct SessionKeyPairs(Arc<dashmap::DashMap<EncodedPoint, (SigningKey, Uuid)>>);

impl SessionKeyPairs {
    pub fn create(self, pubkey: &VerifyingKey) -> (VerifyingKey, Uuid) {
        let sk = k256::ecdsa::SigningKey::random(&mut rand::rngs::OsRng);
        let pk = sk.verifying_key().to_owned();
        let uuid = Uuid::now_v7();

        self.0.insert(pubkey.to_encoded_point(true), (sk, uuid));

        (pk, uuid)
    }
}

struct AgentService {
    name: String,
    service_handle: JoinHandle<anyhow::Result<()>>,
}
