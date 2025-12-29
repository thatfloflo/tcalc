use std::fmt::Display;
use std::ops::{Deref, DerefMut};

use crate::core::tokens::Token;
use crate::core::values::Value;

pub struct Ast {
    _vec: Vec<AstNode>,
    _level: usize,
}

impl Ast {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn level(&self) -> usize {
        self._level
    }

    pub fn push(&mut self, mut item: AstNode) {
        if item.has_subtree() {
            item.subtree.as_mut().unwrap().relevel_from(self._level);
        }
        self._vec.push(item)
    }

    pub fn push_token(&mut self, token: Token) {
        self._vec.push(AstNode::new_from_token(token))
    }

    pub fn push_subtree(&mut self, token: Token, mut subtree: Ast) {
        subtree.relevel_from(self._level + 1);
        self._vec.push(AstNode::new_with_subtree(token, subtree))
    }

    pub fn last(&self) -> Option<&AstNode> {
        self._vec.last()
    }

    pub fn iter(&self) -> impl Iterator<Item = &AstNode> {
        self._vec.iter()
    }

    pub fn len(&self) -> usize {
        self._vec.len()
    }

    pub fn relevel_from(&mut self, base_level: usize) {
        self._level = base_level;
        for node in self._vec.iter_mut() {
            if node.has_subtree() {
                node.subtree.as_mut().unwrap().relevel_from(base_level + 1);
            }
        }
    }
}

impl Default for Ast {
    fn default() -> Self {
        Self {
            _vec: Vec::new(),
            _level: 0,
        }
    }
}

impl Display for Ast {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut formatted = String::new();
        let indent = "    ".repeat(self._level);
        for item in &self._vec {
            formatted.push_str(format!("{:2} {}{}\n", self._level, indent, item).as_str());
        }
        formatted.pop(); // drop last newline
        write!(f, "{}", formatted)
    }
}

impl IntoIterator for Ast {
    type Item = AstNode;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self._vec.into_iter()
    }
}

impl Deref for Ast {
    type Target = Vec<AstNode>;

    fn deref(&self) -> &Self::Target {
        &self._vec
    }
}

impl DerefMut for Ast {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self._vec
    }
}

impl From<&mut Vec<AstNode>> for Ast {
    fn from(value: &mut Vec<AstNode>) -> Self {
        let mut tree = Self::new();
        tree.append(value);
        tree
    }
}

impl From<Vec<AstNode>> for Ast {
    fn from(value: Vec<AstNode>) -> Self {
        let mut tree = Self::new();
        for node in value {
            tree.push(node);
        }
        tree
    }
}

impl From<AstNode> for Ast {
    fn from(value: AstNode) -> Self {
        let mut tree = Self::new();
        tree.push(value);
        tree
    }
}

pub struct AstNode {
    pub token: Token,
    pub subtree: Option<Ast>,
    pub value: Option<Value>,
}

impl AstNode {
    pub fn new_from_token(token: Token) -> Self {
        Self {
            token: token,
            subtree: None,
            value: None,
        }
    }

    pub fn new_with_subtree(token: Token, subtree: Ast) -> Self {
        Self {
            token: token,
            subtree: Some(subtree),
            value: None,
        }
    }

    pub fn has_subtree(&self) -> bool {
        self.subtree.is_some()
    }

    pub fn set_subtree(&mut self, subtree: Ast) {
        self.subtree = Some(subtree);
    }
}

impl Display for AstNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Err(e) = write!(f, "- {}", self.token) {
            return Err(e);
        }
        match &self.value {
            None => {}
            Some(value) => {
                if let Err(e) = write!(f, " -> {}", value) {
                    return Err(e);
                }
            }
        }
        match &self.subtree {
            None => {}
            Some(tree) => {
                if let Err(e) = write!(f, "\n{}", tree) {
                    return Err(e);
                }
            }
        }
        write!(f, "")
    }
}
