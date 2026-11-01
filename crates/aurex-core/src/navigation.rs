//! Bounded session-only navigation model shared by graphical frontends.
use std::{collections::BTreeSet, path::PathBuf};
const HISTORY_LIMIT: usize = 64;
const TAB_LIMIT: usize = 32;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tab {
    history: Vec<PathBuf>,
    cursor: usize,
}
impl Tab {
    fn new(path: PathBuf) -> Self {
        Self {
            history: vec![path],
            cursor: 0,
        }
    }
    pub fn path(&self) -> &PathBuf {
        &self.history[self.cursor]
    }
    pub fn can_back(&self) -> bool {
        self.cursor > 0
    }
    pub fn can_forward(&self) -> bool {
        self.cursor + 1 < self.history.len()
    }
    fn navigate(&mut self, path: PathBuf) {
        if self.path() == &path {
            return;
        }
        self.history.truncate(self.cursor + 1);
        self.history.push(path);
        if self.history.len() > HISTORY_LIMIT {
            self.history.remove(0);
        }
        self.cursor = self.history.len() - 1;
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaneState {
    pub tabs: Vec<Tab>,
    pub active: usize,
    pub selection: BTreeSet<PathBuf>,
    pub filter: String,
    pub show_hidden: bool,
    pub descending: bool,
    pub offset: usize,
}
impl Default for PaneState {
    fn default() -> Self {
        Self {
            tabs: vec![Tab::new(PathBuf::new())],
            active: 0,
            selection: BTreeSet::new(),
            filter: String::new(),
            show_hidden: false,
            descending: false,
            offset: 0,
        }
    }
}
impl PaneState {
    pub fn tab(&self) -> &Tab {
        &self.tabs[self.active]
    }
    pub fn path(&self) -> PathBuf {
        self.tab().path().clone()
    }
    fn clear_view(&mut self) {
        self.selection.clear();
        self.offset = 0;
    }
    pub fn navigate(&mut self, path: PathBuf) {
        self.tabs[self.active].navigate(path);
        self.clear_view();
    }
    pub fn back(&mut self) {
        if self.tab().can_back() {
            self.tabs[self.active].cursor -= 1;
            self.clear_view();
        }
    }
    pub fn forward(&mut self) {
        if self.tab().can_forward() {
            self.tabs[self.active].cursor += 1;
            self.clear_view();
        }
    }
    pub fn parent(&mut self) {
        if let Some(parent) = self.path().parent() {
            self.navigate(parent.to_owned());
        }
    }
    pub fn new_tab(&mut self) {
        if self.tabs.len() < TAB_LIMIT {
            self.tabs.push(Tab::new(self.path()));
            self.active = self.tabs.len() - 1;
            self.clear_view();
        }
    }
    pub fn close_tab(&mut self) {
        if self.tabs.len() > 1 {
            self.tabs.remove(self.active);
            self.active = self.active.min(self.tabs.len() - 1);
            self.clear_view();
        }
    }
    pub fn activate(&mut self, index: usize) {
        if index < self.tabs.len() {
            self.active = index;
            self.clear_view();
        }
    }
    pub fn toggle(&mut self, path: PathBuf) {
        if !self.selection.remove(&path) {
            self.selection.insert(path);
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn history_branches_and_root_parent_is_safe() {
        let mut p = PaneState::default();
        p.parent();
        assert_eq!(p.path(), PathBuf::new());
        p.navigate("a".into());
        p.navigate("a/b".into());
        p.back();
        assert_eq!(p.path(), PathBuf::from("a"));
        p.navigate("c".into());
        assert!(!p.tab().can_forward());
        p.back();
        p.forward();
        assert_eq!(p.path(), PathBuf::from("c"));
    }
    #[test]
    fn tabs_have_independent_history_and_cannot_all_be_closed() {
        let mut p = PaneState::default();
        p.navigate("a".into());
        p.new_tab();
        p.navigate("b".into());
        p.activate(0);
        assert_eq!(p.path(), PathBuf::from("a"));
        p.close_tab();
        assert_eq!(p.path(), PathBuf::from("b"));
        p.close_tab();
        assert_eq!(p.tabs.len(), 1);
    }
    #[test]
    fn navigation_clears_selection_and_paging() {
        let mut p = PaneState::default();
        p.toggle("x".into());
        p.offset = 500;
        p.navigate("dir".into());
        assert!(p.selection.is_empty());
        assert_eq!(p.offset, 0);
    }
    #[test]
    fn history_and_tabs_are_bounded() {
        let mut p = PaneState::default();
        for i in 0..200 {
            p.navigate(i.to_string().into());
            p.new_tab();
        }
        assert!(p.tabs.len() <= TAB_LIMIT);
        for t in &p.tabs {
            assert!(t.history.len() <= HISTORY_LIMIT);
        }
    }
}
