use serde::{Deserialize, Serialize};

/// Item names ordered from most to least recently used.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(transparent)]
pub struct RecentItems(Vec<String>);

impl RecentItems {
    pub fn record(&mut self, name: String) {
        if self.0.first() == Some(&name) {
            return;
        }
        if let Some(index) = self.0.iter().position(|item| item == &name) {
            self.0.remove(index);
        }
        self.0.insert(0, name);
    }

    /// Returns recorded items first, followed by unseen items in their source order.
    pub fn apply<T: Clone>(&self, available: &[T], name: impl Fn(&T) -> &str) -> Vec<T> {
        let mut ordered = self
            .0
            .iter()
            .filter_map(|recent| available.iter().find(|item| name(item) == recent))
            .cloned()
            .collect::<Vec<_>>();
        ordered.extend(
            available
                .iter()
                .filter(|item| !self.0.iter().any(|recent| recent == name(item)))
                .cloned(),
        );
        ordered
    }
}

#[cfg(test)]
mod tests {
    use super::RecentItems;

    #[test]
    fn records_most_recent_first_without_duplicates() {
        let mut recent = RecentItems::default();
        recent.record("build".into());
        recent.record("test".into());
        recent.record("build".into());

        assert_eq!(recent.0, ["build", "test"]);
    }

    #[test]
    fn applies_recent_order_then_preserves_unseen_source_order() {
        let mut recent = RecentItems::default();
        recent.record("third".into());
        recent.record("second".into());
        let available = ["first", "second", "third", "fourth"];

        assert_eq!(
            recent.apply(&available, |item| item),
            ["second", "third", "first", "fourth"]
        );
    }

    #[test]
    fn ignores_recorded_items_that_are_unavailable() {
        let mut recent = RecentItems::default();
        recent.record("removed".into());
        recent.record("second".into());

        assert_eq!(
            recent.apply(&["first", "second"], |item| item),
            ["second", "first"]
        );
    }
}
