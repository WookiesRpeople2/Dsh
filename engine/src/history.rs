use readline::HistoryNavigation;

#[derive(Debug, Clone, Default)]
pub struct History {
    pub entries: Vec<String>,
    pub index: Option<usize>,
    pub pending: String,
}

impl History {
    pub fn push(&mut self, line: &str) {
        let line = line.trim_end_matches(['\r', '\n']);
        if !line.trim().is_empty() {
            self.entries.push(line.to_string());
        }
        self.index = None;
        self.pending.clear();
    }

    pub fn up(&mut self, current: &str) -> Option<&str> {
        if self.entries.is_empty() {
            return None;
        }
        if self.index.is_none() {
            self.pending = current.trim_end_matches(['\r', '\n']).to_string();
        }
        let i = self
            .index
            .map_or(self.entries.len() - 1, |i| i.saturating_sub(1));
        self.index = Some(i);
        Some(&self.entries[i])
    }

    pub fn down(&mut self) -> Option<&str> {
        let i = self.index?;
        if i + 1 >= self.entries.len() {
            self.index = None;
            Some(&self.pending)
        } else {
            self.index = Some(i + 1);
            Some(&self.entries[i + 1])
        }
    }
}

impl HistoryNavigation for History {
    fn previous(&mut self, current: &str) -> Option<String> {
        self.up(current).map(str::to_owned)
    }

    fn next(&mut self) -> Option<String> {
        self.down().map(str::to_owned)
    }
}
