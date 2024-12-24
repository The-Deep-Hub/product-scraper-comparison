use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Store {
    LeroyMerlin,
    Bauhaus,
    Obramat,
} 