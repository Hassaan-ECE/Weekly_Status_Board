use weekly_status_board::history::History;
use weekly_status_board::model::{BoardDocument, Column};

#[test]
fn undo_restores_cleared_done() {
    let mut board = BoardDocument::empty();
    let (p, _) = board.add_project(Column::Done);
    board.set_project_name(&p, "IRHX");
    let mut history = History::new(board.clone());
    board.clear_done();
    history.push(board.clone());
    assert!(board.tasks.is_empty());
    board = history.undo(board).expect("undo");
    assert_eq!(board.tasks.len(), 1);
    board = history.redo(board).expect("redo");
    assert!(board.tasks.is_empty());
}
