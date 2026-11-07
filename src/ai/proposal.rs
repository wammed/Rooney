use crate::editor::buffer::TextBuffer;
use crate::snippet::session::char_to_line_col;
use ropey::Rope;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditProposal {
    pub proposal_id: usize,
    pub tab_id: usize,
    pub base_revision: usize,
    pub target_start_char: usize,
    pub target_end_char: usize,
    pub original_text: String,
    pub proposed_text: String,
    pub prompt_summary: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProposalValidationError {
    TabMismatch,
    RevisionMismatch,
    InvalidRange,
    ContentMismatch,
}

impl std::fmt::Display for ProposalValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TabMismatch => write!(f, "Proposal target tab does not match active tab"),
            Self::RevisionMismatch => write!(f, "Document was modified after proposal was generated"),
            Self::InvalidRange => write!(f, "Target text range is outside current buffer bounds"),
            Self::ContentMismatch => write!(f, "Target text does not match proposal original content"),
        }
    }
}

impl std::error::Error for ProposalValidationError {}

impl EditProposal {
    pub fn new(
        proposal_id: usize,
        tab_id: usize,
        base_revision: usize,
        target_start_char: usize,
        target_end_char: usize,
        original_text: String,
        proposed_text: String,
        prompt_summary: String,
    ) -> Self {
        Self {
            proposal_id,
            tab_id,
            base_revision,
            target_start_char,
            target_end_char,
            original_text,
            proposed_text,
            prompt_summary,
        }
    }

    /// Strictly validates that this proposal can be safely applied to `rope` at `current_revision`.
    pub fn validate(
        &self,
        tab_id: usize,
        current_revision: usize,
        rope: &Rope,
    ) -> Result<(), ProposalValidationError> {
        if self.tab_id != tab_id {
            return Err(ProposalValidationError::TabMismatch);
        }

        if self.base_revision != current_revision {
            return Err(ProposalValidationError::RevisionMismatch);
        }

        if self.target_start_char > self.target_end_char || self.target_end_char > rope.len_chars() {
            return Err(ProposalValidationError::InvalidRange);
        }

        let current_text = rope
            .slice(self.target_start_char..self.target_end_char)
            .to_string();
        if current_text != self.original_text {
            return Err(ProposalValidationError::ContentMismatch);
        }

        Ok(())
    }

    /// Atomically applies the proposal to the `buffer` within an undo transaction.
    pub fn apply(
        &self,
        tab_id: usize,
        buffer: &mut TextBuffer,
    ) -> Result<(), ProposalValidationError> {
        self.validate(tab_id, buffer.revision, &buffer.rope)?;

        buffer.begin_transaction();

        let start_pos = char_to_line_col(&buffer.rope, self.target_start_char);
        let end_pos = char_to_line_col(&buffer.rope, self.target_end_char);

        buffer.cursor = start_pos;
        buffer.selection_anchor = Some(end_pos);

        buffer.delete_selection();
        buffer.insert_str(&self.proposed_text);

        buffer.end_transaction();
        Ok(())
    }
}

