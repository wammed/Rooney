use rooney::editor::buffer::TextBuffer;

#[test]
fn test_transaction_single_undo_reverts_multiple_edits() {
    let mut buf = TextBuffer::new("Hello World");
    assert_eq!(buf.undo_stack_len(), 0);

    // トランザクション開始
    buf.begin_transaction();
    assert!(buf.is_in_transaction());

    // 1. " World" を選択して削除
    buf.cursor = (0, 11);
    buf.selection_anchor = Some((0, 5));
    buf.delete_selection();
    assert_eq!(buf.full_text(), "Hello");

    // 2. ", Rooney Editor!" を挿入
    buf.insert_str(", Rooney Editor!");
    assert_eq!(buf.full_text(), "Hello, Rooney Editor!");

    // 3. 改行と新しい行を追加
    buf.insert_str("\nSecond line");
    assert_eq!(buf.full_text(), "Hello, Rooney Editor!\nSecond line");

    // トランザクション終了
    buf.end_transaction();
    assert!(!buf.is_in_transaction());

    // 複数回の変更を行ったが、undo_stack には 1 つのスナップショットのみ積まれているはず
    assert_eq!(buf.undo_stack_len(), 1);

    // 1 回の undo() で開始前の "Hello World" に戻る
    buf.undo();
    assert_eq!(buf.full_text(), "Hello World");
    assert_eq!(buf.cursor, (0, 0));
    assert_eq!(buf.undo_stack_len(), 0);
    assert_eq!(buf.redo_stack_len(), 1);

    // redo() でトランザクション完了後の状態に復元される
    buf.redo();
    assert_eq!(buf.full_text(), "Hello, Rooney Editor!\nSecond line");
    assert_eq!(buf.undo_stack_len(), 1);
    assert_eq!(buf.redo_stack_len(), 0);
}

#[test]
fn test_nested_transactions() {
    let mut buf = TextBuffer::new("initial");
    buf.cursor = (0, 7);

    buf.begin_transaction();
    buf.insert_str(" A");

    // ネストしたトランザクション
    buf.begin_transaction();
    buf.insert_str(" B");
    buf.end_transaction();

    assert!(buf.is_in_transaction());
    assert_eq!(buf.undo_stack_len(), 0);

    buf.insert_str(" C");
    buf.end_transaction();

    assert!(!buf.is_in_transaction());
    assert_eq!(buf.undo_stack_len(), 1);
    assert_eq!(buf.full_text(), "initial A B C");

    // 1 回の undo で initial に戻る
    buf.undo();
    assert_eq!(buf.full_text(), "initial");
}

#[test]
fn test_empty_transaction() {
    let mut buf = TextBuffer::new("no change");

    buf.begin_transaction();
    // 何も編集しない
    buf.end_transaction();

    // 変更がないため undo_stack は増えない
    assert_eq!(buf.undo_stack_len(), 0);
}

#[test]
fn test_cancel_transaction_rollback() {
    let mut buf = TextBuffer::new("original text");
    buf.cursor = (0, 8);

    buf.begin_transaction();
    buf.insert_str("DANGEROUS EDIT");
    assert_eq!(buf.full_text(), "originalDANGEROUS EDIT text");

    // ロールバック
    buf.cancel_transaction();
    assert!(!buf.is_in_transaction());
    assert_eq!(buf.full_text(), "original text");
    assert_eq!(buf.cursor, (0, 8));
    assert_eq!(buf.undo_stack_len(), 0);
}
