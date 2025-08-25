use casper_types::{AsymmetricType, Deploy as _Deploy};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use wasm_bindgen::prelude::*;

#[derive(Debug, Deserialize, Clone, Serialize)]
#[wasm_bindgen]
pub struct Deploy(_Deploy);

#[wasm_bindgen]
#[deprecated(note = "prefer Transaction type")]
#[allow(deprecated)]
impl Deploy {
    pub fn add_signature(&self, public_key: &str, signature: &str) -> Deploy {
        // Serialize the existing approvals to JSON
        let casper_deploy: _Deploy = self.0.clone();
        let existing_approvals_json = casper_deploy
            .approvals()
            .iter()
            .map(|approval| {
                json!({
                    "signer": approval.signer().to_hex(),
                    "signature": approval.signature().to_hex(),
                })
            })
            .collect::<Vec<_>>();

        // Create JSON object for the new approval
        let new_approval_json = json!({
            "signer": public_key,
            "signature": signature,
        });

        // Append the new approval to existing approvals
        let mut all_approvals_json = existing_approvals_json;
        all_approvals_json.push(new_approval_json);

        // Convert the approvals JSON back to string
        let updated_approvals_str = serde_json::to_string(&all_approvals_json)
            .expect("Failed to serialize updated approvals JSON");

        // Replace the approvals field in the original deploy JSON string
        let mut deploy_json: Value = serde_json::from_str(&self.to_json_string().unwrap())
            .expect("Failed to deserialize deploy JSON");
        deploy_json["approvals"] = serde_json::from_str(&updated_approvals_str)
            .expect("Failed to deserialize updated approvals JSON");

        // Convert the updated deploy JSON back to a Deploy struct
        let updated_deploy: Deploy =
            serde_json::from_value(deploy_json).expect("Failed to deserialize updated deploy JSON");

        updated_deploy
    }
}

#[deprecated(note = "prefer Transaction type")]
#[allow(deprecated)]
impl Deploy {
    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string(&self.0).map_err(|e| format!("Error serializing deploy to JSON: {e}"))
    }

    pub fn from_json_string(json_str: &str) -> Result<Deploy, String> {
        serde_json::from_str(json_str)
            .map_err(|e| format!("Error deserializing deploy from JSON: {e}"))
    }
}

impl From<Deploy> for _Deploy {
    fn from(deploy: Deploy) -> Self {
        deploy.0
    }
}

impl From<_Deploy> for Deploy {
    fn from(deploy: _Deploy) -> Self {
        Deploy(deploy)
    }
}
