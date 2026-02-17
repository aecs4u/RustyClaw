//! Context compaction for long conversations.
//!
//! Enables indefinitely long conversations by intelligently compacting message
//! history when it grows too large. Supports multiple strategies:
//!
//! - **SlidingWindow**: Keep first N + last N messages (simplest, fastest)
//! - **Summarize**: LLM generates executive summary of old messages
//! - **Importance**: Score messages by semantic relevance, keep important ones
//! - **Hybrid**: Combine strategies for optimal results
//!
//! ## Configuration Example
//!
//! ```toml
//! [context_compaction]
//! enabled = true
//! strategy = "sliding_window"  # or "summarize", "importance", "hybrid"
//! max_messages = 100
//! keep_recent = 20  # For sliding window
//! keep_initial = 5  # For sliding window
//! ```

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::gateway::ChatMessage;

/// Context compaction strategy.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CompactionStrategy {
    /// Keep first N + last N messages, discard middle
    SlidingWindow,
    /// LLM summarizes old messages into compact form
    Summarize,
    /// Score messages by importance, keep high-value ones
    Importance,
    /// Combine multiple strategies
    Hybrid,
}

impl Default for CompactionStrategy {
    fn default() -> Self {
        Self::SlidingWindow
    }
}

/// Context compaction configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactionConfig {
    /// Whether compaction is enabled
    #[serde(default)]
    pub enabled: bool,

    /// Compaction strategy to use
    #[serde(default)]
    pub strategy: CompactionStrategy,

    /// Maximum messages before compaction triggers
    #[serde(default = "CompactionConfig::default_max_messages")]
    pub max_messages: usize,

    /// Number of recent messages to keep (for sliding window)
    #[serde(default = "CompactionConfig::default_keep_recent")]
    pub keep_recent: usize,

    /// Number of initial messages to keep (for sliding window)
    #[serde(default = "CompactionConfig::default_keep_initial")]
    pub keep_initial: usize,

    /// Minimum importance score to keep message (0.0-1.0, for importance strategy)
    #[serde(default = "CompactionConfig::default_importance_threshold")]
    pub importance_threshold: f64,
}

impl CompactionConfig {
    fn default_max_messages() -> usize {
        100
    }

    fn default_keep_recent() -> usize {
        20
    }

    fn default_keep_initial() -> usize {
        5
    }

    fn default_importance_threshold() -> f64 {
        0.5
    }
}

impl Default for CompactionConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            strategy: CompactionStrategy::default(),
            max_messages: Self::default_max_messages(),
            keep_recent: Self::default_keep_recent(),
            keep_initial: Self::default_keep_initial(),
            importance_threshold: Self::default_importance_threshold(),
        }
    }
}

/// Result of a compaction operation.
#[derive(Debug)]
pub struct CompactionResult {
    /// Number of messages before compaction
    pub before: usize,
    /// Number of messages after compaction
    pub after: usize,
    /// Whether compaction was performed
    pub compacted: bool,
    /// Strategy used
    pub strategy: CompactionStrategy,
}

impl CompactionResult {
    /// Calculate compression ratio (0.0-1.0, higher = more compression)
    pub fn compression_ratio(&self) -> f64 {
        if self.before == 0 {
            return 0.0;
        }
        1.0 - (self.after as f64 / self.before as f64)
    }
}

/// Context compaction engine.
pub struct CompactionEngine {
    config: CompactionConfig,
}

impl CompactionEngine {
    /// Create a new compaction engine with the given configuration.
    pub fn new(config: CompactionConfig) -> Self {
        Self { config }
    }

    /// Check if compaction should be triggered for the given message count.
    pub fn should_compact(&self, message_count: usize) -> bool {
        self.config.enabled && message_count > self.config.max_messages
    }

    /// Compact a message history using the configured strategy.
    ///
    /// Returns the compacted message list and statistics.
    pub fn compact(&self, messages: Vec<ChatMessage>) -> Result<(Vec<ChatMessage>, CompactionResult)> {
        let before = messages.len();

        if !self.should_compact(before) {
            return Ok((
                messages,
                CompactionResult {
                    before,
                    after: before,
                    compacted: false,
                    strategy: self.config.strategy.clone(),
                },
            ));
        }

        let (compacted, strategy) = match self.config.strategy {
            CompactionStrategy::SlidingWindow => {
                (self.compact_sliding_window(messages)?, CompactionStrategy::SlidingWindow)
            }
            CompactionStrategy::Summarize => {
                (self.compact_summarize(messages)?, CompactionStrategy::Summarize)
            }
            CompactionStrategy::Importance => {
                (self.compact_importance(messages)?, CompactionStrategy::Importance)
            }
            CompactionStrategy::Hybrid => {
                (self.compact_hybrid(messages)?, CompactionStrategy::Hybrid)
            }
        };

        let after = compacted.len();

        Ok((
            compacted,
            CompactionResult {
                before,
                after,
                compacted: true,
                strategy,
            },
        ))
    }

    /// Sliding window compaction: keep first N + last M messages.
    fn compact_sliding_window(&self, mut messages: Vec<ChatMessage>) -> Result<Vec<ChatMessage>> {
        let total = messages.len();
        let keep_initial = self.config.keep_initial.min(total);
        let keep_recent = self.config.keep_recent.min(total);

        // If keeping initial + recent covers everything, no compaction needed
        if keep_initial + keep_recent >= total {
            return Ok(messages);
        }

        // Extract initial messages
        let mut compacted: Vec<ChatMessage> = messages.drain(0..keep_initial).collect();

        // Add a summary marker message
        let removed_count = total - keep_initial - keep_recent;
        compacted.push(ChatMessage {
            role: "system".to_string(),
            content: format!(
                "[Context compacted: {} messages removed from history using sliding window strategy]",
                removed_count
            ),
            ..Default::default()
        });

        // Add recent messages
        let recent_start = messages.len() - keep_recent;
        compacted.extend(messages.drain(recent_start..));

        Ok(compacted)
    }

    /// Summarization-based compaction:
    /// keep initial + recent messages and replace the middle section with
    /// a generated summary block.
    fn compact_summarize(&self, mut messages: Vec<ChatMessage>) -> Result<Vec<ChatMessage>> {
        let total = messages.len();
        let keep_initial = self.config.keep_initial.min(total);
        let keep_recent = self.config.keep_recent.min(total);

        if keep_initial + keep_recent >= total {
            return Ok(messages);
        }

        let mut compacted: Vec<ChatMessage> = messages.drain(0..keep_initial).collect();
        let middle_count = total - keep_initial - keep_recent;
        let middle_messages: Vec<ChatMessage> = messages.drain(0..middle_count).collect();
        let summary = self.summarize_messages(&middle_messages, 8);

        compacted.push(ChatMessage {
            role: "system".to_string(),
            content: format!(
                "[Context summary: {} earlier messages compacted]\n{}",
                middle_count, summary
            ),
            ..Default::default()
        });

        compacted.extend(messages.into_iter());
        Ok(compacted)
    }

    /// Calculate importance score for a message (0.0-1.0).
    ///
    /// Heuristic scoring based on:
    /// - Message length (longer = potentially more important)
    /// - Role (user messages often more important than assistant acknowledgments)
    /// - Presence of tool calls
    /// - Keywords indicating importance
    fn calculate_importance(&self, message: &ChatMessage) -> f64 {
        let mut score = 0.0;

        // Base score by role
        match message.role.as_str() {
            "user" => score += 0.4,
            "assistant" => score += 0.3,
            "system" => score += 0.5,
            _ => score += 0.2,
        }

        // Length factor (normalize to 0-0.3 range)
        let length_score = (message.content.len() as f64 / 1000.0).min(1.0) * 0.3;
        score += length_score;

        // Tool calls are important
        if message.tool_calls.is_some() || message.tool_call_id.is_some() {
            score += 0.2;
        }

        // Media attachments indicate importance
        if message.media.is_some() {
            score += 0.1;
        }

        // Keyword detection
        let content_lower = message.content.to_lowercase();
        let important_keywords = [
            "error", "warning", "important", "critical", "fix", "bug",
            "todo", "action", "decide", "confirm", "approve",
        ];

        for keyword in &important_keywords {
            if content_lower.contains(keyword) {
                score += 0.15;
                break;
            }
        }

        // Clamp to 0.0-1.0 range
        score.clamp(0.0, 1.0)
    }

    /// Importance-based compaction: score messages and keep high-value ones.
    fn compact_importance(&self, messages: Vec<ChatMessage>) -> Result<Vec<ChatMessage>> {
        let total = messages.len();
        if total == 0 {
            return Ok(Vec::new());
        }

        let keep_initial = self.config.keep_initial.min(total);
        let keep_recent = self.config.keep_recent.min(total.saturating_sub(keep_initial));
        let recent_start = total.saturating_sub(keep_recent);
        let target_count = self.config.max_messages.max(1) * 2 / 3; // Keep ~67%

        let mut keep_indices: HashSet<usize> = HashSet::new();

        for idx in 0..keep_initial {
            keep_indices.insert(idx);
        }
        for idx in recent_start..total {
            keep_indices.insert(idx);
        }

        let mut middle_scored: Vec<(usize, f64)> = (keep_initial..recent_start)
            .map(|idx| (idx, self.calculate_importance(&messages[idx])))
            .collect();
        middle_scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let remaining_budget = target_count.saturating_sub(keep_indices.len());
        for (idx, score) in middle_scored.into_iter().take(remaining_budget) {
            if score >= self.config.importance_threshold {
                keep_indices.insert(idx);
            }
        }

        let kept_count = keep_indices.len();
        let removed = total.saturating_sub(kept_count);

        let mut compacted = Vec::with_capacity(kept_count + usize::from(removed > 0));
        let mut marker_inserted = false;

        for (idx, msg) in messages.into_iter().enumerate() {
            if keep_indices.contains(&idx) {
                compacted.push(msg);
            } else if !marker_inserted {
                compacted.push(ChatMessage {
                    role: "system".to_string(),
                    content: format!(
                        "[Context compacted: {} low-importance messages removed]",
                        removed
                    ),
                    ..Default::default()
                });
                marker_inserted = true;
            }
        }

        Ok(compacted)
    }

    /// Hybrid compaction:
    /// keep initial/recent, retain highest-importance middle messages, and
    /// summarize the rest of the middle context.
    fn compact_hybrid(&self, messages: Vec<ChatMessage>) -> Result<Vec<ChatMessage>> {
        let total = messages.len();
        if total == 0 {
            return Ok(Vec::new());
        }

        let keep_initial = self.config.keep_initial.min(total);
        let keep_recent = self.config.keep_recent.min(total.saturating_sub(keep_initial));
        let recent_start = total.saturating_sub(keep_recent);

        if keep_initial + keep_recent >= total {
            return Ok(messages);
        }

        let mut middle_scored: Vec<(usize, f64)> = (keep_initial..recent_start)
            .map(|idx| (idx, self.calculate_importance(&messages[idx])))
            .collect();
        middle_scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let middle_keep_target = ((recent_start - keep_initial) / 4).max(1);
        let middle_keep: HashSet<usize> = middle_scored
            .into_iter()
            .take(middle_keep_target)
            .filter(|(_, score)| *score >= self.config.importance_threshold)
            .map(|(idx, _)| idx)
            .collect();

        let mut omitted_middle = Vec::new();
        let mut compacted = Vec::new();
        let mut inserted_summary = false;

        for (idx, msg) in messages.into_iter().enumerate() {
            let in_initial = idx < keep_initial;
            let in_recent = idx >= recent_start;
            let keep_mid = middle_keep.contains(&idx);

            if in_initial || in_recent || keep_mid {
                if !inserted_summary && !omitted_middle.is_empty() {
                    let summary = self.summarize_messages(&omitted_middle, 6);
                    compacted.push(ChatMessage {
                        role: "system".to_string(),
                        content: format!(
                            "[Context summary: {} messages compacted in hybrid mode]\n{}",
                            omitted_middle.len(),
                            summary
                        ),
                        ..Default::default()
                    });
                    inserted_summary = true;
                }
                compacted.push(msg);
            } else {
                omitted_middle.push(msg);
            }
        }

        if !omitted_middle.is_empty() && !inserted_summary {
            let summary = self.summarize_messages(&omitted_middle, 6);
            compacted.push(ChatMessage {
                role: "system".to_string(),
                content: format!(
                    "[Context summary: {} messages compacted in hybrid mode]\n{}",
                    omitted_middle.len(),
                    summary
                ),
                ..Default::default()
            });
        }

        Ok(compacted)
    }

    fn summarize_messages(&self, messages: &[ChatMessage], max_points: usize) -> String {
        if messages.is_empty() {
            return "- No significant context.".to_string();
        }

        let mut scored: Vec<(usize, f64)> = messages
            .iter()
            .enumerate()
            .map(|(idx, msg)| (idx, self.calculate_importance(msg)))
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let mut chosen: Vec<usize> = scored
            .into_iter()
            .take(max_points.max(1))
            .map(|(idx, _)| idx)
            .collect();
        chosen.sort_unstable();

        let mut lines = Vec::new();
        for idx in chosen {
            let msg = &messages[idx];
            let mut snippet = msg.content.replace('\n', " ").trim().to_string();
            if snippet.chars().count() > 140 {
                snippet = snippet.chars().take(140).collect::<String>();
                snippet.push_str("...");
            }
            lines.push(format!("- {}: {}", msg.role, snippet));
        }
        lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_message(role: &str, content: &str) -> ChatMessage {
        ChatMessage {
            role: role.to_string(),
            content: content.to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn test_should_compact() {
        let config = CompactionConfig {
            enabled: true,
            max_messages: 10,
            ..Default::default()
        };

        let engine = CompactionEngine::new(config);

        assert!(!engine.should_compact(5));
        assert!(!engine.should_compact(10));
        assert!(engine.should_compact(11));
        assert!(engine.should_compact(100));
    }

    #[test]
    fn test_sliding_window_basic() {
        let config = CompactionConfig {
            enabled: true,
            strategy: CompactionStrategy::SlidingWindow,
            max_messages: 10,
            keep_initial: 2,
            keep_recent: 3,
            ..Default::default()
        };

        let engine = CompactionEngine::new(config);

        let messages: Vec<_> = (0..20)
            .map(|i| create_test_message("user", &format!("Message {}", i)))
            .collect();

        let (compacted, result) = engine.compact(messages).unwrap();

        assert!(result.compacted);
        assert_eq!(result.before, 20);
        // Should have: 2 initial + 1 marker + 3 recent = 6 messages
        assert_eq!(compacted.len(), 6);
        assert_eq!(compacted[0].content, "Message 0");
        assert_eq!(compacted[1].content, "Message 1");
        assert!(compacted[2].content.contains("Context compacted"));
        assert_eq!(compacted[3].content, "Message 17");
        assert_eq!(compacted[4].content, "Message 18");
        assert_eq!(compacted[5].content, "Message 19");
    }

    #[test]
    fn test_no_compaction_when_disabled() {
        let config = CompactionConfig {
            enabled: false,
            max_messages: 10,
            ..Default::default()
        };

        let engine = CompactionEngine::new(config);

        let messages: Vec<_> = (0..20)
            .map(|i| create_test_message("user", &format!("Message {}", i)))
            .collect();

        let (compacted, result) = engine.compact(messages).unwrap();

        assert!(!result.compacted);
        assert_eq!(result.before, 20);
        assert_eq!(result.after, 20);
        assert_eq!(compacted.len(), 20);
    }

    #[test]
    fn test_importance_scoring() {
        let config = CompactionConfig::default();
        let engine = CompactionEngine::new(config);

        // User message with error keyword
        let msg1 = create_test_message("user", "There's an error in the code");
        assert!(engine.calculate_importance(&msg1) > 0.5);

        // Short assistant acknowledgment
        let msg2 = create_test_message("assistant", "OK");
        assert!(engine.calculate_importance(&msg2) < 0.5);

        // System message
        let msg3 = create_test_message("system", "Configuration loaded");
        assert!(engine.calculate_importance(&msg3) > 0.5);

        // Long detailed message
        let msg4 = create_test_message(
            "user",
            &"x".repeat(1000), // Long message
        );
        assert!(engine.calculate_importance(&msg4) > 0.6);
    }

    #[test]
    fn test_compression_ratio() {
        let result = CompactionResult {
            before: 100,
            after: 25,
            compacted: true,
            strategy: CompactionStrategy::SlidingWindow,
        };

        assert_eq!(result.compression_ratio(), 0.75); // 75% compression
    }

    #[test]
    fn test_summarize_strategy_used() {
        let config = CompactionConfig {
            enabled: true,
            strategy: CompactionStrategy::Summarize,
            max_messages: 6,
            keep_initial: 1,
            keep_recent: 2,
            ..Default::default()
        };
        let engine = CompactionEngine::new(config);
        let messages: Vec<_> = (0..10)
            .map(|i| create_test_message("user", &format!("Message {}", i)))
            .collect();

        let (compacted, result) = engine.compact(messages).unwrap();
        assert_eq!(result.strategy, CompactionStrategy::Summarize);
        assert!(compacted.iter().any(|m| m.content.contains("Context summary")));
    }

    #[test]
    fn test_hybrid_strategy_used() {
        let config = CompactionConfig {
            enabled: true,
            strategy: CompactionStrategy::Hybrid,
            max_messages: 8,
            keep_initial: 1,
            keep_recent: 2,
            importance_threshold: 0.1,
            ..Default::default()
        };
        let engine = CompactionEngine::new(config);
        let messages: Vec<_> = (0..14)
            .map(|i| create_test_message("user", &format!("Important message {}", i)))
            .collect();

        let (compacted, result) = engine.compact(messages).unwrap();
        assert_eq!(result.strategy, CompactionStrategy::Hybrid);
        assert!(compacted.len() < result.before);
    }
}
