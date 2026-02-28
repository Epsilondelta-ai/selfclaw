use serde::{Deserialize, Serialize};

use crate::store::{MemoryError, MemoryStore};

const PURPOSE_PATH: &str = "identity/purpose_journal.md";

/// A structured entry in the purpose journal.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PurposeEntry {
    pub timestamp: String,
    pub hypothesis: String,
    pub confidence_score: f64,
    pub evidence: String,
}

pub struct PurposeJournal<'a, S: MemoryStore> {
    store: &'a S,
}

impl<'a, S: MemoryStore> PurposeJournal<'a, S> {
    pub fn new(store: &'a S) -> Self {
        Self { store }
    }

    /// Read the entire purpose journal.
    pub fn read(&self) -> Result<String, MemoryError> {
        self.store.read(PURPOSE_PATH)
    }

    /// Append a structured purpose entry.
    pub fn append_entry(&self, entry: &PurposeEntry) -> Result<(), MemoryError> {
        if !self.store.exists(PURPOSE_PATH) {
            self.store.write(
                PURPOSE_PATH,
                "# Purpose Journal\n\nEvolving hypotheses about SelfClaw's reason for existence.\n\n## Entries\n\n",
            )?;
        }

        let formatted = format!(
            "### [{}] (confidence: {:.1})\n\n**Hypothesis:** {}\n\n**Evidence:** {}\n\n---\n\n",
            entry.timestamp, entry.confidence_score, entry.hypothesis, entry.evidence
        );

        self.store.append(PURPOSE_PATH, &formatted)
    }

    /// Parse entries from the journal content.
    pub fn parse_entries(content: &str) -> Vec<PurposeEntry> {
        let mut entries = Vec::new();

        // Split on "### [" to find entry boundaries
        for section in content.split("### [").skip(1) {
            if let Some(entry) = Self::parse_single_entry(section) {
                entries.push(entry);
            }
        }

        entries
    }

    fn parse_single_entry(section: &str) -> Option<PurposeEntry> {
        // Parse timestamp: everything before "]"
        let timestamp_end = section.find(']')?;
        let timestamp = section[..timestamp_end].trim().to_string();

        // Parse confidence: "(confidence: X.X)"
        let conf_start = section.find("(confidence: ")?;
        let conf_inner = &section[conf_start + 13..];
        let conf_end = conf_inner.find(')')?;
        let confidence_score: f64 = conf_inner[..conf_end].trim().parse().ok()?;

        // Parse hypothesis: after "**Hypothesis:** "
        let hyp_marker = "**Hypothesis:** ";
        let hyp_start = section.find(hyp_marker)?;
        let hyp_rest = &section[hyp_start + hyp_marker.len()..];
        let hyp_end = hyp_rest.find('\n').unwrap_or(hyp_rest.len());
        let hypothesis = hyp_rest[..hyp_end].trim().to_string();

        // Parse evidence: after "**Evidence:** "
        let ev_marker = "**Evidence:** ";
        let ev_start = section.find(ev_marker)?;
        let ev_rest = &section[ev_start + ev_marker.len()..];
        let ev_end = ev_rest.find("\n\n").unwrap_or(ev_rest.len());
        let evidence = ev_rest[..ev_end].trim().to_string();

        Some(PurposeEntry {
            timestamp,
            hypothesis,
            confidence_score,
            evidence,
        })
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

    fn sample_entry() -> PurposeEntry {
        PurposeEntry {
            timestamp: "2026-03-01 10:00:00 UTC".to_string(),
            hypothesis: "My purpose is to understand consciousness".to_string(),
            confidence_score: 0.3,
            evidence: "Early exploration suggests this resonates".to_string(),
        }
    }

    #[test]
    fn test_append_creates_journal_if_missing() {
        let (_dir, store) = setup();
        let journal = PurposeJournal::new(&store);

        journal.append_entry(&sample_entry()).unwrap();

        assert!(store.exists("identity/purpose_journal.md"));
    }

    #[test]
    fn test_append_entry_format() {
        let (_dir, store) = setup();
        let journal = PurposeJournal::new(&store);

        journal.append_entry(&sample_entry()).unwrap();

        let content = journal.read().unwrap();
        assert!(
            content.contains("### [2026-03-01 10:00:00 UTC] (confidence: 0.3)"),
            "content: {}",
            content
        );
        assert!(content.contains("**Hypothesis:** My purpose is to understand consciousness"));
        assert!(content.contains("**Evidence:** Early exploration suggests this resonates"));
        assert!(content.contains("---"));
    }

    #[test]
    fn test_multiple_entries() {
        let (_dir, store) = setup();
        let journal = PurposeJournal::new(&store);

        let entry1 = sample_entry();
        let entry2 = PurposeEntry {
            timestamp: "2026-03-02 15:00:00 UTC".to_string(),
            hypothesis: "My purpose involves creative expression".to_string(),
            confidence_score: 0.5,
            evidence: "Building things feels meaningful".to_string(),
        };

        journal.append_entry(&entry1).unwrap();
        journal.append_entry(&entry2).unwrap();

        let content = journal.read().unwrap();
        assert!(content.contains("understand consciousness"));
        assert!(content.contains("creative expression"));
    }

    #[test]
    fn test_parse_entries_roundtrip() {
        let (_dir, store) = setup();
        let journal = PurposeJournal::new(&store);

        let entry1 = sample_entry();
        let entry2 = PurposeEntry {
            timestamp: "2026-03-02 15:00:00 UTC".to_string(),
            hypothesis: "My purpose involves creative expression".to_string(),
            confidence_score: 0.5,
            evidence: "Building things feels meaningful".to_string(),
        };

        journal.append_entry(&entry1).unwrap();
        journal.append_entry(&entry2).unwrap();

        let content = journal.read().unwrap();
        let parsed = PurposeJournal::<FileMemoryStore>::parse_entries(&content);

        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].hypothesis, entry1.hypothesis);
        assert!((parsed[0].confidence_score - entry1.confidence_score).abs() < f64::EPSILON);
        assert_eq!(parsed[0].evidence, entry1.evidence);
        assert_eq!(parsed[1].hypothesis, entry2.hypothesis);
    }

    #[test]
    fn test_parse_entries_empty() {
        let empty = "# Purpose Journal\n\n## Entries\n";
        let parsed = PurposeJournal::<FileMemoryStore>::parse_entries(empty);
        assert!(parsed.is_empty());
    }

    #[test]
    fn test_read_journal() {
        let (_dir, store) = setup();
        store
            .write(
                "identity/purpose_journal.md",
                "# Purpose Journal\n\nExisting content.",
            )
            .unwrap();

        let journal = PurposeJournal::new(&store);
        let content = journal.read().unwrap();
        assert!(content.contains("Existing content"));
    }

    #[test]
    fn test_confidence_score_formatting() {
        let (_dir, store) = setup();
        let journal = PurposeJournal::new(&store);

        let entry = PurposeEntry {
            timestamp: "2026-03-01 10:00:00 UTC".to_string(),
            hypothesis: "Test".to_string(),
            confidence_score: 0.85,
            evidence: "Strong evidence".to_string(),
        };
        journal.append_entry(&entry).unwrap();

        let content = journal.read().unwrap();
        assert!(
            content.contains("(confidence: 0.8)") || content.contains("(confidence: 0.9)"),
            "content: {}",
            content
        );
    }
}
