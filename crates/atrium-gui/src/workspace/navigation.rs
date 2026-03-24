//! Back/forward navigation for worktree switching.

/// Navigation history stack.
#[derive(Debug, Default)]
pub struct NavigationStack {
    back: Vec<usize>,
    forward: Vec<usize>,
    current: Option<usize>,
}

impl NavigationStack {
    /// Navigate to a new index, pushing current onto back stack.
    pub fn go_to(&mut self, index: usize) {
        if let Some(current) = self.current {
            self.back.push(current);
        }
        self.current = Some(index);
        self.forward.clear();
    }

    /// Go back. Returns the new current index, if any.
    pub fn go_back(&mut self) -> Option<usize> {
        let prev = self.back.pop()?;
        if let Some(current) = self.current {
            self.forward.push(current);
        }
        self.current = Some(prev);
        Some(prev)
    }

    /// Go forward. Returns the new current index, if any.
    pub fn go_forward(&mut self) -> Option<usize> {
        let next = self.forward.pop()?;
        if let Some(current) = self.current {
            self.back.push(current);
        }
        self.current = Some(next);
        Some(next)
    }

    /// Current index.
    pub fn current(&self) -> Option<usize> {
        self.current
    }

    /// Whether back navigation is possible.
    pub fn can_go_back(&self) -> bool {
        !self.back.is_empty()
    }

    /// Whether forward navigation is possible.
    pub fn can_go_forward(&self) -> bool {
        !self.forward.is_empty()
    }
}
