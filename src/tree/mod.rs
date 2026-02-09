mod file_node;

pub use file_node::FileNode;

use anyhow::Result;
use ignore::WalkBuilder;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub struct FileTree {
    root: PathBuf,
    nodes: Vec<FileNode>,
    pub show_hidden: bool,
    max_depth: usize,
    selected: usize,
    expanded: HashSet<PathBuf>,
    search_matches: Vec<usize>,
    current_match: usize,
    offset: usize,
}

impl FileTree {
    pub fn new(root: &Path, show_hidden: bool, max_depth: usize) -> Result<Self> {
        let mut tree = Self {
            root: root.to_path_buf(),
            nodes: Vec::new(),
            show_hidden,
            max_depth,
            selected: 0,
            expanded: HashSet::new(),
            search_matches: Vec::new(),
            current_match: 0,
            offset: 0,
        };

        // Start with root expanded
        tree.expanded.insert(root.to_path_buf());
        tree.rebuild_visible_nodes()?;

        Ok(tree)
    }

    pub fn root_path(&self) -> &Path {
        &self.root
    }

    pub fn nodes(&self) -> &[FileNode] {
        &self.nodes
    }

    pub fn selected(&self) -> usize {
        self.selected
    }

    pub fn offset(&self) -> usize {
        self.offset
    }

    pub fn set_offset(&mut self, offset: usize) {
        self.offset = offset;
    }

    pub fn is_expanded(&self, path: &Path) -> bool {
        self.expanded.contains(path)
    }

    fn rebuild_visible_nodes(&mut self) -> Result<()> {
        self.nodes.clear();
        self.build_tree(&self.root.clone(), 0)?;
        Ok(())
    }

    fn build_tree(&mut self, path: &Path, depth: usize) -> Result<()> {
        if depth > self.max_depth {
            return Ok(());
        }

        let walker = WalkBuilder::new(path)
            .hidden(!self.show_hidden)
            .git_ignore(true)
            .git_global(true)
            .git_exclude(true)
            .max_depth(Some(1))
            .sort_by_file_name(|a, b| {
                // Alphabetical sort by file name (directory-first sorting
                // is handled after walking via entry metadata)
                a.cmp(b)
            })
            .build();

        for entry in walker.flatten() {
            let entry_path = entry.path();

            // Skip the root itself when iterating
            if entry_path == path && depth > 0 {
                continue;
            }

            // At depth 0, only process the root entry itself.
            // Children will be added by the recursive call below.
            if depth == 0 && entry_path != path {
                continue;
            }

            let is_dir = entry_path.is_dir();
            let name = entry_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| entry_path.to_string_lossy().to_string());

            // Skip hidden files if not showing them
            if !self.show_hidden && name.starts_with('.') && depth > 0 {
                continue;
            }

            let node = FileNode::new(entry_path.to_path_buf(), name, depth, is_dir);
            self.nodes.push(node);

            // Recursively add children if expanded
            if is_dir && self.expanded.contains(entry_path) {
                self.build_tree(entry_path, depth + 1)?;
            }
        }

        Ok(())
    }

    pub fn select_next(&mut self) {
        if self.selected < self.nodes.len().saturating_sub(1) {
            self.selected += 1;
        }
    }

    pub fn select_previous(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    pub fn select_first(&mut self) {
        self.selected = 0;
    }

    pub fn select_last(&mut self) {
        self.selected = self.nodes.len().saturating_sub(1);
    }

    pub fn page_up(&mut self, amount: usize) {
        self.selected = self.selected.saturating_sub(amount);
    }

    pub fn page_down(&mut self, amount: usize) {
        self.selected = (self.selected + amount).min(self.nodes.len().saturating_sub(1));
    }

    pub fn toggle_expand(&mut self) {
        if let Some(node) = self.nodes.get(self.selected) {
            if node.is_dir {
                let path = node.path.clone();
                if self.expanded.contains(&path) {
                    self.expanded.remove(&path);
                } else {
                    self.expanded.insert(path);
                }
                let _ = self.rebuild_visible_nodes();
            }
        }
    }

    pub fn toggle_or_open(&mut self) -> Option<PathBuf> {
        if let Some(node) = self.nodes.get(self.selected) {
            let path = node.path.clone();
            if node.is_dir {
                if self.expanded.contains(&path) {
                    // Already expanded, return for cd
                    return Some(path);
                } else {
                    self.expanded.insert(path.clone());
                    let _ = self.rebuild_visible_nodes();
                    return None;
                }
            } else {
                // File - return for opening/referencing
                return Some(path);
            }
        }
        None
    }

    pub fn collapse_or_parent(&mut self) {
        if let Some(node) = self.nodes.get(self.selected) {
            if node.is_dir && self.expanded.contains(&node.path) {
                // Collapse current directory
                self.expanded.remove(&node.path.clone());
                let _ = self.rebuild_visible_nodes();
            } else if let Some(parent) = node.path.parent() {
                // Go to parent
                if let Some(idx) = self.nodes.iter().position(|n| n.path == parent) {
                    self.selected = idx;
                }
            }
        }
    }

    pub fn refresh(&mut self) {
        let _ = self.rebuild_visible_nodes();
        self.selected = self.selected.min(self.nodes.len().saturating_sub(1));
    }

    pub fn refresh_path(&mut self, _path: &Path) {
        // For now, just do a full refresh
        // Could be optimized to only refresh the affected subtree
        self.refresh();
    }

    pub fn toggle_hidden(&mut self) {
        self.show_hidden = !self.show_hidden;
        self.refresh();
    }

    pub fn search(&mut self, query: &str) {
        self.search_matches.clear();
        self.current_match = 0;

        if query.is_empty() {
            return;
        }

        let query_lower = query.to_lowercase();
        for (idx, node) in self.nodes.iter().enumerate() {
            if node.name.to_lowercase().contains(&query_lower) {
                self.search_matches.push(idx);
            }
        }

        if !self.search_matches.is_empty() {
            self.selected = self.search_matches[0];
        }
    }

    pub fn search_next(&mut self) {
        if self.search_matches.is_empty() {
            return;
        }
        self.current_match = (self.current_match + 1) % self.search_matches.len();
        self.selected = self.search_matches[self.current_match];
    }

    pub fn search_prev(&mut self) {
        if self.search_matches.is_empty() {
            return;
        }
        self.current_match = if self.current_match == 0 {
            self.search_matches.len() - 1
        } else {
            self.current_match - 1
        };
        self.selected = self.search_matches[self.current_match];
    }
}
