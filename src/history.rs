use crate::model::BoardDocument;

pub struct History {
    undo: Vec<BoardDocument>,
    redo: Vec<BoardDocument>,
}

impl History {
    pub fn new(current: BoardDocument) -> Self {
        Self {
            undo: vec![current],
            redo: Vec::new(),
        }
    }

    pub fn push(&mut self, current: BoardDocument) {
        self.undo.push(current);
        self.redo.clear();
    }

    pub fn undo(&mut self, current: BoardDocument) -> Option<BoardDocument> {
        if self.undo.len() < 2 {
            return None;
        }
        self.redo.push(current);
        self.undo.pop();
        self.undo.last().cloned()
    }

    pub fn redo(&mut self, current: BoardDocument) -> Option<BoardDocument> {
        let next = self.redo.pop()?;
        self.undo.push(next.clone());
        let _ = current;
        Some(next)
    }
}
