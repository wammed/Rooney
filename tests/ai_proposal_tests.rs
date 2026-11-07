use rooney::ai::proposal::{EditProposal, ProposalValidationError};
use rooney::editor::buffer::TextBuffer;
use rooney::editor::pane::EditorTab;

#[test]
fn test_ai_proposal_validation_success_and_apply() {
    let mut buffer = TextBuffer::new("Hello, world! This is a test.");
    let tab_id = 1;
    let base_rev = buffer.revision;

    // Propose replacing "world" with "Rooney"
    let target_start = 7;
    let target_end = 12;
    assert_eq!(&buffer.rope.slice(target_start..target_end).to_string(), "world");

    let proposal = EditProposal::new(
        1,
        tab_id,
        base_rev,
        target_start,
        target_end,
        "world".to_string(),
        "Rooney".to_string(),
        "Improve greeting".to_string(),
    );

    // Validate
    assert!(proposal.validate(tab_id, buffer.revision, &buffer.rope).is_ok());

    // Apply
    let res = proposal.apply(tab_id, &mut buffer);
    assert!(res.is_ok());
    assert_eq!(buffer.full_text(), "Hello, Rooney! This is a test.");
}

#[test]
fn test_ai_proposal_undo_single_transaction() {
    let mut buffer = TextBuffer::new("The quick brown fox jumps over the lazy dog.");
    let initial_text = buffer.full_text();
    let tab_id = 1;
    let base_rev = buffer.revision;

    // Replace "brown fox" (10..19) with "cosmic terminal"
    let target_start = 10;
    let target_end = 19;
    assert_eq!(&buffer.rope.slice(target_start..target_end).to_string(), "brown fox");

    let proposal = EditProposal::new(
        2,
        tab_id,
        base_rev,
        target_start,
        target_end,
        "brown fox".to_string(),
        "cosmic terminal".to_string(),
        "Modernize terminology".to_string(),
    );

    proposal.apply(tab_id, &mut buffer).expect("Apply proposal failed");
    assert_eq!(buffer.full_text(), "The quick cosmic terminal jumps over the lazy dog.");

    // Single Undo step restores initial state completely
    buffer.undo();
    assert_eq!(buffer.full_text(), initial_text);

    // Redo step reapplies the proposal
    buffer.redo();
    assert_eq!(buffer.full_text(), "The quick cosmic terminal jumps over the lazy dog.");
}

#[test]
fn test_ai_proposal_safety_guard_revision_mismatch() {
    let mut buffer = TextBuffer::new("Function calculate_total() { return 0; }");
    let tab_id = 1;
    let base_rev = buffer.revision;

    let proposal = EditProposal::new(
        3,
        tab_id,
        base_rev,
        29,
        37,
        "return 0;".to_string(),
        "return price * quantity;".to_string(),
        "Implement calculation".to_string(),
    );

    // User types something before proposal is accepted, bumping revision
    buffer.cursor = (0, 0);
    buffer.insert_str("// TODO\n");
    assert_ne!(buffer.revision, base_rev);

    // Validation must fail with RevisionMismatch
    let err = proposal.validate(tab_id, buffer.revision, &buffer.rope).unwrap_err();
    assert_eq!(err, ProposalValidationError::RevisionMismatch);

    // Apply must also fail safely without corrupting the buffer
    let apply_err = proposal.apply(tab_id, &mut buffer).unwrap_err();
    assert_eq!(apply_err, ProposalValidationError::RevisionMismatch);
    assert!(buffer.full_text().contains("// TODO\nFunction calculate_total()"));
}

#[test]
fn test_ai_proposal_safety_guard_tab_mismatch() {
    let mut buffer = TextBuffer::new("Sample text in tab 1");
    let target_tab = 1;
    let wrong_tab = 2;

    let proposal = EditProposal::new(
        4,
        target_tab,
        buffer.revision,
        0,
        6,
        "Sample".to_string(),
        "Modified".to_string(),
        "Summary".to_string(),
    );

    let err = proposal.validate(wrong_tab, buffer.revision, &buffer.rope).unwrap_err();
    assert_eq!(err, ProposalValidationError::TabMismatch);

    let apply_err = proposal.apply(wrong_tab, &mut buffer).unwrap_err();
    assert_eq!(apply_err, ProposalValidationError::TabMismatch);
}

#[test]
fn test_ai_proposal_safety_guard_content_mismatch() {
    let mut buffer = TextBuffer::new("Target content was changed");
    let tab_id = 1;

    let proposal = EditProposal::new(
        5,
        tab_id,
        buffer.revision,
        0,
        6,
        "Origin".to_string(), // Expected "Origin" but buffer has "Target"
        "NewText".to_string(),
        "Summary".to_string(),
    );

    let err = proposal.validate(tab_id, buffer.revision, &buffer.rope).unwrap_err();
    assert_eq!(err, ProposalValidationError::ContentMismatch);

    let apply_err = proposal.apply(tab_id, &mut buffer).unwrap_err();
    assert_eq!(apply_err, ProposalValidationError::ContentMismatch);
}

#[test]
fn test_ai_proposal_safety_guard_invalid_range() {
    let mut buffer = TextBuffer::new("Short");
    let tab_id = 1;

    let proposal = EditProposal::new(
        6,
        tab_id,
        buffer.revision,
        0,
        100, // Out of bounds
        "Out of bounds".to_string(),
        "Replacement".to_string(),
        "Summary".to_string(),
    );

    let err = proposal.validate(tab_id, buffer.revision, &buffer.rope).unwrap_err();
    assert_eq!(err, ProposalValidationError::InvalidRange);

    let apply_err = proposal.apply(tab_id, &mut buffer).unwrap_err();
    assert_eq!(apply_err, ProposalValidationError::InvalidRange);
}

#[test]
fn test_editor_tab_active_proposal_lifecycle() {
    let mut tab = EditorTab::new(42, "test.md");
    tab.buffer.insert_str("Line 1\nLine 2\nLine 3");
    let tab_id = tab.id;
    let base_rev = tab.buffer.revision;

    let proposal = EditProposal::new(
        10,
        tab_id,
        base_rev,
        7,
        13,
        "Line 2".to_string(),
        "Second Line Modified".to_string(),
        "Rewrite Line 2".to_string(),
    );

    // Set proposal as active
    tab.active_proposal = Some(proposal.clone());
    assert!(tab.active_proposal.is_some());

    // Apply active proposal
    let res = tab.apply_active_proposal();
    assert!(res.is_ok());
    assert!(tab.active_proposal.is_none());
    assert_eq!(tab.buffer.full_text(), "Line 1\nSecond Line Modified\nLine 3");

    // Propose again and reject
    let rev2 = tab.buffer.revision;
    let proposal2 = EditProposal::new(
        11,
        tab_id,
        rev2,
        0,
        6,
        "Line 1".to_string(),
        "First Line".to_string(),
        "Rewrite Line 1".to_string(),
    );
    tab.active_proposal = Some(proposal2);
    tab.reject_active_proposal();
    assert!(tab.active_proposal.is_none());
    assert_eq!(tab.buffer.full_text(), "Line 1\nSecond Line Modified\nLine 3");

    // Propose again and verify that direct user edit auto-discards old proposal
    let rev3 = tab.buffer.revision;
    let proposal3 = EditProposal::new(
        12,
        tab_id,
        rev3,
        0,
        6,
        "Line 1".to_string(),
        "First Line".to_string(),
        "Rewrite Line 1".to_string(),
    );
    tab.active_proposal = Some(proposal3);
    assert!(tab.active_proposal.is_some());

    // User edits document
    tab.buffer.insert_str("User typing...");
    tab.on_content_changed();
    assert!(tab.active_proposal.is_none(), "Old proposal must be automatically discarded when content changes");
}
