use chrono::{NaiveDate, Utc};

use crate::store::{MemoryError, MemoryStore};

pub struct EpisodicLogger<'a, S: MemoryStore> {
    store: &'a S,
}

impl<'a, S: MemoryStore> EpisodicLogger<'a, S> {
    pub fn new(store: &'a S) -> Self {
        Self { store }
    }

    /// Append a timestamped entry to today's episodic log.
    pub fn log(&self, content: &str) -> Result<(), MemoryError> {
        let now = Utc::now();
        self.log_for_date(now.format("%Y-%m-%d").to_string().as_str(), content, &now.format("%H:%M:%S UTC").to_string())
    }

    /// Append a timestamped entry to a specific date's log.
    pub fn log_for_date(
        &self,
        date: &str,
        content: &str,
        time_str: &str,
    ) -> Result<(), MemoryError> {
        let path = format!("episodic/{}.md", date);

        // If the file doesn't exist, create it with a header.
        if !self.store.exists(&path) {
            self.store.write(
                &path,
                &format!("# Episodic Log: {}\n\n", date),
            )?;
        }

        let entry = format!("## [{}]\n\n{}\n\n---\n\n", time_str, content);
        self.store.append(&path, &entry)
    }

    /// Read the log for a specific date.
    pub fn read_date(&self, date: &str) -> Result<String, MemoryError> {
        let path = format!("episodic/{}.md", date);
        self.store.read(&path)
    }

    /// Read today's log.
    pub fn read_today(&self) -> Result<String, MemoryError> {
        let today = Utc::now().format("%Y-%m-%d").to_string();
        self.read_date(&today)
    }

    /// Parse a date string into NaiveDate (for validation).
    pub fn parse_date(date: &str) -> Option<NaiveDate> {
        NaiveDate::parse_from_str(date, "%Y-%m-%d").ok()
    }
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::FileMemoryStore;
    use tempfile::TempDir;

    fn setup() -> (TempDir, FileMemoryStore) {
        let dir = TempDir::new().unwrap();
        let store = FileMemoryStore::new(dir.path());
        (dir, store)
    }

    #[test]
    fn test_log_creates_file_with_header() {
        let (_dir, store) = setup();
        let logger = EpisodicLogger::new(&store);

        logger
            .log_for_date("2026-03-01", "First entry", "10:00:00 UTC")
            .unwrap();

        let content = store.read("episodic/2026-03-01.md").unwrap();
        assert!(content.starts_with("# Episodic Log: 2026-03-01"));
    }

    #[test]
    fn test_log_entry_format() {
        let (_dir, store) = setup();
        let logger = EpisodicLogger::new(&store);

        logger
            .log_for_date("2026-03-01", "Something happened", "14:30:00 UTC")
            .unwrap();

        let content = store.read("episodic/2026-03-01.md").unwrap();
        assert!(content.contains("## [14:30:00 UTC]"), "content: {}", content);
        assert!(content.contains("Something happened"), "content: {}", content);
        assert!(content.contains("---"), "content: {}", content);
    }

    #[test]
    fn test_multiple_entries_appended() {
        let (_dir, store) = setup();
        let logger = EpisodicLogger::new(&store);

        logger
            .log_for_date("2026-03-01", "Entry one", "10:00:00 UTC")
            .unwrap();
        logger
            .log_for_date("2026-03-01", "Entry two", "11:00:00 UTC")
            .unwrap();

        let content = store.read("episodic/2026-03-01.md").unwrap();
        assert!(content.contains("Entry one"));
        assert!(content.contains("Entry two"));
        assert!(content.contains("[10:00:00 UTC]"));
        assert!(content.contains("[11:00:00 UTC]"));
    }

    #[test]
    fn test_different_dates_separate_files() {
        let (_dir, store) = setup();
        let logger = EpisodicLogger::new(&store);

        logger
            .log_for_date("2026-03-01", "Day one", "10:00:00 UTC")
            .unwrap();
        logger
            .log_for_date("2026-03-02", "Day two", "10:00:00 UTC")
            .unwrap();

        assert!(store.exists("episodic/2026-03-01.md"));
        assert!(store.exists("episodic/2026-03-02.md"));

        let day1 = store.read("episodic/2026-03-01.md").unwrap();
        let day2 = store.read("episodic/2026-03-02.md").unwrap();
        assert!(day1.contains("Day one") && !day1.contains("Day two"));
        assert!(day2.contains("Day two") && !day2.contains("Day one"));
    }

    #[test]
    fn test_read_date() {
        let (_dir, store) = setup();
        let logger = EpisodicLogger::new(&store);

        logger
            .log_for_date("2026-03-01", "Test", "12:00:00 UTC")
            .unwrap();

        let content = logger.read_date("2026-03-01").unwrap();
        assert!(content.contains("Test"));
    }

    #[test]
    fn test_parse_date_valid() {
        assert!(EpisodicLogger::<FileMemoryStore>::parse_date("2026-03-01").is_some());
    }

    #[test]
    fn test_parse_date_invalid() {
        assert!(EpisodicLogger::<FileMemoryStore>::parse_date("not-a-date").is_none());
    }
}
