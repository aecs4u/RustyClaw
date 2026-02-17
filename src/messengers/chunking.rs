//! Per-channel message chunking utilities.

/// Conservative chunking limits per messenger/channel type.
#[derive(Debug, Clone, Copy)]
pub struct ChunkingPolicy {
    pub max_chars: usize,
    pub inter_chunk_delay_ms: u64,
}

/// Return chunking policy for a messenger type.
pub fn policy_for_messenger(messenger_type: &str) -> ChunkingPolicy {
    match messenger_type {
        // Telegram documented hard limit.
        "telegram" => ChunkingPolicy {
            max_chars: 4096,
            inter_chunk_delay_ms: 25,
        },
        // Discord hard limit.
        "discord" => ChunkingPolicy {
            max_chars: 2000,
            inter_chunk_delay_ms: 25,
        },
        // Use conservative limits for webhook/bot integrations.
        "slack" | "google-chat" | "mattermost" | "lark" | "feishu" => ChunkingPolicy {
            max_chars: 3900,
            inter_chunk_delay_ms: 25,
        },
        "irc" => ChunkingPolicy {
            max_chars: 380,
            inter_chunk_delay_ms: 20,
        },
        "line" => ChunkingPolicy {
            max_chars: 4900,
            inter_chunk_delay_ms: 25,
        },
        // Default fallback.
        _ => ChunkingPolicy {
            max_chars: 3500,
            inter_chunk_delay_ms: 25,
        },
    }
}

/// Split a message into chunks according to a max char limit.
///
/// The splitter prefers paragraph, line, and word boundaries.
pub fn chunk_message(content: &str, max_chars: usize) -> Vec<String> {
    if content.is_empty() || max_chars == 0 {
        return vec![String::new()];
    }

    if content.chars().count() <= max_chars {
        return vec![content.to_string()];
    }

    let mut chunks = Vec::new();
    let mut remaining = content.trim();

    while !remaining.is_empty() {
        if remaining.chars().count() <= max_chars {
            chunks.push(remaining.to_string());
            break;
        }

        let hard_idx = byte_idx_after_n_chars(remaining, max_chars);
        let prefix = &remaining[..hard_idx];

        // Try natural boundaries first.
        let mut cut_idx = prefix.rfind("\n\n");
        if cut_idx.is_none() {
            cut_idx = prefix.rfind('\n');
        }
        if cut_idx.is_none() {
            cut_idx = prefix.rfind(". ");
        }
        if cut_idx.is_none() {
            cut_idx = prefix.rfind(' ');
        }

        // Avoid tiny chunks when a boundary is too early.
        let min_reasonable = byte_idx_after_n_chars(remaining, max_chars / 2);
        let final_idx = match cut_idx {
            Some(i) if i >= min_reasonable => i,
            _ => hard_idx,
        };

        let chunk = remaining[..final_idx].trim_end();
        if !chunk.is_empty() {
            chunks.push(chunk.to_string());
        }
        remaining = remaining[final_idx..].trim_start();
    }

    if chunks.is_empty() {
        chunks.push(content.to_string());
    }
    chunks
}

fn byte_idx_after_n_chars(s: &str, n: usize) -> usize {
    if n == 0 {
        return 0;
    }
    s.char_indices()
        .nth(n)
        .map(|(idx, _)| idx)
        .unwrap_or_else(|| s.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_message_no_split() {
        let chunks = chunk_message("hello", 10);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], "hello");
    }

    #[test]
    fn test_chunk_message_splits_large_text() {
        let text = "a".repeat(9500);
        let chunks = chunk_message(&text, 4096);
        assert!(chunks.len() >= 3);
        assert!(chunks.iter().all(|c| c.chars().count() <= 4096));
        assert_eq!(chunks.concat(), text);
    }

    #[test]
    fn test_policy_for_messenger() {
        assert_eq!(policy_for_messenger("telegram").max_chars, 4096);
        assert_eq!(policy_for_messenger("discord").max_chars, 2000);
        assert_eq!(policy_for_messenger("irc").max_chars, 380);
    }
}

