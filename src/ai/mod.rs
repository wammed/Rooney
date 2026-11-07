pub mod ollama;
pub mod proposal;

pub use ollama::{AiStatus, ChatMessage, ChatRole, ChatStreamEvent, OllamaClient, OllamaError};
pub use proposal::{EditProposal, ProposalValidationError};
