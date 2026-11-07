use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum BridgeLogEvent {
    ServerStarted { address: String },
    ServerStopped,
    ClientConnected { address: String },
    ClientDisconnected { address: String },
    VariablesRequested { count: usize },
    DataPreviewRequested { limit: usize },
    DatasetLoaded { rows: usize, cols: usize },
    SyntaxExecuted { preview: String, count: usize },
    CommandExecuted { command_id: String },
    OutputNoteAdded { title: String },
    TabSwitched { tab: String },
    Error { message: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BridgeLogEntry {
    pub timestamp: String,
    pub event: BridgeLogEvent,
}

impl BridgeLogEntry {
    pub fn new(event: BridgeLogEvent) -> Self {
        Self {
            timestamp: current_time_string(),
            event,
        }
    }

    pub fn with_timestamp(timestamp: impl Into<String>, event: BridgeLogEvent) -> Self {
        Self {
            timestamp: timestamp.into(),
            event,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeLogRing {
    entries: VecDeque<BridgeLogEntry>,
    capacity: usize,
}

impl Default for BridgeLogRing {
    fn default() -> Self {
        Self::new(100)
    }
}

impl BridgeLogRing {
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: VecDeque::with_capacity(capacity),
            capacity: capacity.max(1),
        }
    }

    pub fn push(&mut self, event: BridgeLogEvent) {
        if self.entries.len() >= self.capacity {
            let _ = self.entries.pop_front();
        }
        self.entries.push_back(BridgeLogEntry::new(event));
    }

    pub fn push_entry(&mut self, entry: BridgeLogEntry) {
        if self.entries.len() >= self.capacity {
            let _ = self.entries.pop_front();
        }
        self.entries.push_back(entry);
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn entries(&self) -> &VecDeque<BridgeLogEntry> {
        &self.entries
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

pub fn current_time_string() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let rem_secs = secs % 86400;
    let hours = rem_secs / 3600;
    let minutes = (rem_secs % 3600) / 60;
    let seconds = rem_secs % 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_buffer_capacity() {
        let mut ring = BridgeLogRing::new(3);
        ring.push(BridgeLogEvent::ServerStarted {
            address: "127.0.0.1:18493".to_string(),
        });
        ring.push(BridgeLogEvent::TabSwitched {
            tab: "data".to_string(),
        });
        ring.push(BridgeLogEvent::OutputNoteAdded {
            title: "Test".to_string(),
        });
        assert_eq!(ring.len(), 3);

        ring.push(BridgeLogEvent::ServerStopped);
        assert_eq!(ring.len(), 3);
        assert_eq!(
            ring.entries().back().map(|e| &e.event),
            Some(&BridgeLogEvent::ServerStopped)
        );
        assert_eq!(
            ring.entries().front().map(|e| &e.event),
            Some(&BridgeLogEvent::TabSwitched {
                tab: "data".to_string()
            })
        );
    }
}
