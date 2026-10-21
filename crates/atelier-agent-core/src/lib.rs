//! Pure Agent primitives. Host authentication, persistence and execution stay outside this crate.
//! See third_party/shellpilot/SOURCES.json for upstream hashes and adaptation details.
pub mod budget;
pub mod schema;
pub mod strict_json;
pub mod tools;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid Agent input: {0}")]
    Invalid(String),
    #[error("Request cannot fit the model context window")]
    BudgetExceeded,
    #[error("Tool call/result sequence is incomplete")]
    ToolProtocol,
}
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreCapabilities {
    pub tool_argument_validation: bool,
    pub context_budget_planning: bool,
}

/// These flags describe compiled primitives, not a connected Agent product.
pub fn capabilities() -> CoreCapabilities {
    CoreCapabilities {
        tool_argument_validation: true,
        context_budget_planning: true,
    }
}
