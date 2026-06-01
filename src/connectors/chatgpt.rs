use std::fs;
use std::path::{Path, PathBuf};

use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit},
};
use anyhow::{Context, Result};
use base64::prelude::*;
use serde_json::Value;
use walkdir::WalkDir;

use super::{
    Connector, DetectionResult, DiscoveredSourceFile, DiscoveredSourceRole, NormalizedConversation,
    NormalizedMessage, ScanContext, ScanRoot, file_modified_since, franken_detection_for_connector,
    parse_timestamp,
};

const NONCE_SIZE: usize = 12;
const TAG_SIZE: usize = 16;
const KEY_SIZE: usize = 32;
const MAX_CONVERSATION_FILE_BYTES: u64 = 100 * 1024 * 1024;

pub struct ChatGptConnector {
    encryption_key: Option<[u8; KEY_SIZE]>,
}

impl Default for ChatGptConnector {
    fn default() -> Self {
        Self::new()
    }
}

impl ChatGptConnector {
    #[must_use]
    pub fn new() -> Self {
        let encryption_key = Self::load_encryption_key();
        if encryption_key.is_some() {
            tracing::info!(
                "chatgpt encryption key loaded, encrypted conversations will be decrypted"
            );
        }
        Self { encryption_key }
    }

    fn load_encryption_key() -> Option<[u8; KEY_SIZE]> {
        if let Ok(key_b64) = dotenvy::var("CHATGPT_ENCRYPTION_KEY") {
            if let Ok(key_bytes) = BASE64_STANDARD.decode(key_b64.trim()) {
                if key_bytes.len() == KEY_SIZE {
                    let mut key = [0u8; KEY_SIZE];
                    key.copy_from_slice(&key_bytes);
                    tracing::debug!(
                        "chatgpt encryption key loaded from CHATGPT_ENCRYPTION_KEY env var"
                    );
                    return Some(key);
                }
                tracing::warn!(
                    "CHATGPT_ENCRYPTION_KEY has wrong length: {} (expected {})",
                    key_bytes.len(),
                    KEY_SIZE
                );
            } else {
                tracing::warn!("CHATGPT_ENCRYPTION_KEY is not valid base64");
            }
        }

        let key_file_paths = [
            dirs::config_dir().map(|p| p.join("cass/chatgpt_key.bin")),
            dirs::home_dir().map(|p| p.join(".config/cass/chatgpt_key.bin")),
            dirs::home_dir().map(|p| p.join(".cass/chatgpt_key.bin")),
        ];

        for path in key_file_paths.iter().flatten() {
            if !path.exists() {
                continue;
            }
            if !Self::chatgpt_key_file_mode_is_safe(path) {
                continue;
            }
            match fs::read(path) {
                Ok(key_bytes) if key_bytes.len() == KEY_SIZE => {
                    let mut key = [0u8; KEY_SIZE];
                    key.copy_from_slice(&key_bytes);
                    tracing::debug!(path = %path.display(), "chatgpt encryption key loaded from file");
                    return Some(key);
                }
                Ok(key_bytes) => {
                    tracing::warn!(
                        path = %path.display(),
                        "chatgpt key file has wrong size: {} (expected {})",
                        key_bytes.len(),
                        KEY_SIZE
                    );
                }
                Err(error) => {
                    tracing::warn!(
                        path = %path.display(),
                        error = %error,
                        "failed to read chatgpt key file"
                    );
                }
            }
        }

        None
    }

    #[cfg(unix)]
    fn chatgpt_key_file_mode_is_safe(path: &Path) -> bool {
        use std::os::unix::fs::MetadataExt;

        let metadata = match fs::symlink_metadata(path) {
            Ok(metadata) => metadata,
            Err(error) => {
                tracing::warn!(
                    path = %path.display(),
                    error = %error,
                    "failed to stat chatgpt key file"
                );
                return false;
            }
        };
        if metadata.file_type().is_symlink() {
            tracing::warn!(
                path = %path.display(),
                "refusing to load chatgpt key file because it is a symlink"
            );
            return false;
        }
        let mode = metadata.mode() & 0o777;
        if mode & 0o077 != 0 {
            tracing::warn!(
                path = %path.display(),
                mode = format!("{mode:#o}"),
                "refusing to load chatgpt key file because group/other permissions are set"
            );
            return false;
        }
        true
    }

    #[cfg(not(unix))]
    fn chatgpt_key_file_mode_is_safe(_path: &Path) -> bool {
        true
    }

    pub fn app_support_dir() -> Option<PathBuf> {
        #[cfg(target_os = "macos")]
        {
            dirs::home_dir().map(|home| home.join("Library/Application Support/com.openai.chat"))
        }
        #[cfg(not(target_os = "macos"))]
        {
            None
        }
    }

    fn find_conversation_dirs(base: &Path) -> Vec<(PathBuf, bool)> {
        let mut dirs = Vec::new();

        if !base.exists() {
            return dirs;
        }

        if base.is_dir()
            && let Some(name) = base.file_name().and_then(|name| name.to_str())
            && name.starts_with("conversations-")
        {
            dirs.push((base.to_path_buf(), Self::dir_is_encrypted(name)));
            return dirs;
        }

        for entry in fs::read_dir(base).into_iter().flatten().flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("");
            if name.starts_with("conversations-") {
                let is_encrypted = Self::dir_is_encrypted(name);
                dirs.push((path, is_encrypted));
            }
        }

        dirs.sort_by(|a, b| a.0.cmp(&b.0));
        dirs
    }

    fn dir_is_encrypted(name: &str) -> bool {
        name.contains("-v2-") || name.contains("-v3-")
    }

    fn looks_like_base(path: &Path) -> bool {
        if !path.is_dir() {
            return false;
        }

        if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("conversations-"))
        {
            return true;
        }

        fs::read_dir(path)
            .map(|entries| {
                entries.flatten().any(|entry| {
                    entry.file_name().to_str().is_some_and(|name| {
                        name.starts_with("conversations-") && entry.path().is_dir()
                    })
                })
            })
            .unwrap_or(false)
    }

    fn append_explicit_roots(roots: &mut Vec<ScanRoot>, root: &ScanRoot) {
        let base = &root.path;
        if Self::looks_like_base(base) {
            roots.push(root.clone());
        }

        let candidates = [
            base.join("com.openai.chat"),
            base.join("Library/Application Support/com.openai.chat"),
            base.join("AppData/Roaming/com.openai.chat"),
        ];
        for candidate in candidates {
            if Self::looks_like_base(&candidate) {
                roots.push(root.with_path(candidate));
            }
        }
    }

    fn conversation_files(dir_path: &Path) -> Vec<PathBuf> {
        let mut files = Vec::new();
        for entry in WalkDir::new(dir_path).max_depth(1).into_iter().flatten() {
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            let ext = path.extension().and_then(|ext| ext.to_str());
            if ext == Some("json") || ext == Some("data") {
                files.push(path.to_path_buf());
            }
        }
        files.sort();
        files
    }

    fn source_roots(ctx: &ScanContext) -> Vec<ScanRoot> {
        let mut roots: Vec<ScanRoot> = if ctx.use_default_detection() {
            if Self::looks_like_base(&ctx.data_dir) {
                vec![ScanRoot::local(ctx.data_dir.clone())]
            } else if let Some(default_base) = Self::app_support_dir() {
                vec![ScanRoot::local(default_base)]
            } else {
                Vec::new()
            }
        } else {
            let mut explicit = Vec::new();
            for root in &ctx.scan_roots {
                Self::append_explicit_roots(&mut explicit, root);
            }
            explicit
        };

        roots.sort_by(|a, b| a.path.cmp(&b.path));
        roots.dedup_by(|a, b| a.path == b.path);
        roots
    }

    fn discover_sources(&self, ctx: &ScanContext) -> Vec<DiscoveredSourceFile> {
        let mut out = Vec::new();
        for root in Self::source_roots(ctx) {
            if !root.path.exists() {
                continue;
            }
            for (dir_path, is_encrypted) in Self::find_conversation_dirs(&root.path) {
                if is_encrypted && self.encryption_key.is_none() {
                    tracing::debug!(
                        path = %dir_path.display(),
                        "chatgpt skipping encrypted directory during discovery because no decryption key is configured"
                    );
                    continue;
                }
                for path in Self::conversation_files(&dir_path) {
                    if !file_modified_since(&path, ctx.since_ts) {
                        continue;
                    }
                    out.push(
                        DiscoveredSourceFile::new(
                            "chatgpt",
                            &root,
                            path,
                            DiscoveredSourceRole::PrimarySessionLog,
                            true,
                        )
                        .with_fs_metadata(),
                    );
                }
            }
        }
        out
    }

    fn decrypt_file(&self, data: &[u8]) -> Result<Vec<u8>> {
        let key = self.encryption_key.ok_or_else(|| {
            anyhow::anyhow!(
                "No encryption key available. Set CHATGPT_ENCRYPTION_KEY or create ~/.config/cass/chatgpt_key.bin."
            )
        })?;
        if data.len() < NONCE_SIZE + TAG_SIZE {
            anyhow::bail!("Encrypted data too short: {} bytes", data.len());
        }

        let nonce = Nonce::from_slice(&data[..NONCE_SIZE]);
        let ciphertext = &data[NONCE_SIZE..];
        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|error| anyhow::anyhow!("Failed to create cipher: {error}"))?;
        cipher
            .decrypt(nonce, ciphertext)
            .map_err(|error| anyhow::anyhow!("Decryption failed: {error}"))
    }

    fn parse_conversation_file(
        &self,
        path: &Path,
        is_encrypted: bool,
    ) -> Result<Option<NormalizedConversation>> {
        if let Ok(metadata) = fs::metadata(path)
            && metadata.len() > MAX_CONVERSATION_FILE_BYTES
        {
            tracing::warn!(
                path = %path.display(),
                size_bytes = metadata.len(),
                "skipping large chatgpt conversation file"
            );
            return Ok(None);
        }

        let content_bytes = fs::read(path).with_context(|| format!("read {}", path.display()))?;
        let content = if is_encrypted {
            let decrypted = self.decrypt_file(&content_bytes)?;
            String::from_utf8(decrypted).with_context(|| {
                format!(
                    "decrypted content is not valid UTF-8 from {}",
                    path.display()
                )
            })?
        } else {
            String::from_utf8(content_bytes)
                .with_context(|| format!("content is not valid UTF-8 from {}", path.display()))?
        };
        let value: Value = serde_json::from_str(&content)
            .with_context(|| format!("parse JSON from {}", path.display()))?;

        let external_id = value
            .get("id")
            .or_else(|| value.get("conversation_id"))
            .and_then(|raw| raw.as_str())
            .or_else(|| path.file_stem().and_then(|stem| stem.to_str()))
            .map(String::from);
        let title = value
            .get("title")
            .and_then(|raw| raw.as_str())
            .map(String::from);
        let mut messages = Vec::new();
        let mut started_at = None;
        let mut ended_at = None;

        if let Some(mapping) = value.get("mapping").and_then(|raw| raw.as_object()) {
            let mut nodes: Vec<(Option<String>, String, &Value)> = Vec::new();
            for (node_id, node) in mapping {
                if let Some(message) = node.get("message") {
                    let parent = node
                        .get("parent")
                        .and_then(|raw| raw.as_str())
                        .map(String::from);
                    nodes.push((parent, node_id.clone(), message));
                }
            }
            nodes.sort_by(|a, b| {
                let ts_a = a.2.get("create_time").and_then(|raw| raw.as_f64());
                let ts_b = b.2.get("create_time").and_then(|raw| raw.as_f64());
                match (ts_a, ts_b) {
                    (Some(a_ts), Some(b_ts)) => a_ts
                        .partial_cmp(&b_ts)
                        .unwrap_or(std::cmp::Ordering::Equal)
                        .then_with(|| a.1.cmp(&b.1)),
                    (Some(_), None) => std::cmp::Ordering::Less,
                    (None, Some(_)) => std::cmp::Ordering::Greater,
                    (None, None) => a.1.cmp(&b.1),
                }
            });

            for (_, _, message) in nodes {
                let role = message
                    .get("author")
                    .and_then(|author| author.get("role"))
                    .and_then(|raw| raw.as_str())
                    .unwrap_or("assistant");
                if role == "system" {
                    continue;
                }

                let Some(content) = mapping_message_content(message) else {
                    continue;
                };
                if content.trim().is_empty() {
                    continue;
                }
                let created_at = message
                    .get("create_time")
                    .and_then(|raw| raw.as_f64())
                    .map(|ts| (ts * 1000.0).round() as i64);
                update_time_bounds(&mut started_at, &mut ended_at, created_at);
                let model = message
                    .get("metadata")
                    .and_then(|metadata| metadata.get("model_slug"))
                    .and_then(|raw| raw.as_str())
                    .map(String::from);
                messages.push(NormalizedMessage {
                    idx: messages.len() as i64,
                    role: role.to_string(),
                    author: model,
                    created_at,
                    content,
                    extra: message.clone(),
                    invocations: Vec::new(),
                    snippets: Vec::new(),
                });
            }
        }

        if messages.is_empty()
            && let Some(raw_messages) = value.get("messages").and_then(|raw| raw.as_array())
        {
            for item in raw_messages {
                let role = item
                    .get("role")
                    .and_then(|raw| raw.as_str())
                    .unwrap_or("assistant");
                if role == "system" {
                    continue;
                }
                let content = item
                    .get("content")
                    .and_then(|raw| raw.as_str())
                    .unwrap_or("");
                if content.trim().is_empty() {
                    continue;
                }
                let created_at = item
                    .get("timestamp")
                    .or_else(|| item.get("create_time"))
                    .and_then(parse_timestamp);
                update_time_bounds(&mut started_at, &mut ended_at, created_at);
                messages.push(NormalizedMessage {
                    idx: messages.len() as i64,
                    role: role.to_string(),
                    author: None,
                    created_at,
                    content: content.to_string(),
                    extra: item.clone(),
                    invocations: Vec::new(),
                    snippets: Vec::new(),
                });
            }
        }

        if messages.is_empty() {
            return Ok(None);
        }

        Ok(Some(NormalizedConversation {
            agent_slug: "chatgpt".to_string(),
            external_id,
            title,
            workspace: None,
            source_path: path.to_path_buf(),
            started_at,
            ended_at,
            metadata: serde_json::json!({
                "source": if is_encrypted { "chatgpt_desktop_encrypted" } else { "chatgpt_desktop" },
                "model": value.get("model").and_then(|raw| raw.as_str()),
                "encrypted": is_encrypted,
            }),
            messages,
        }))
    }
}

impl Connector for ChatGptConnector {
    fn detect(&self) -> DetectionResult {
        franken_detection_for_connector("chatgpt").unwrap_or_else(DetectionResult::not_found)
    }

    fn scan(&self, ctx: &ScanContext) -> Result<Vec<NormalizedConversation>> {
        let mut conversations = Vec::new();
        self.scan_with_callback(ctx, &mut |conversation| {
            conversations.push(conversation);
            Ok(())
        })?;
        Ok(conversations)
    }

    fn supports_streaming_scan(&self) -> bool {
        true
    }

    fn discover_source_files(&self, ctx: &ScanContext) -> Result<Vec<DiscoveredSourceFile>> {
        Ok(self.discover_sources(ctx))
    }

    fn scan_with_callback(
        &self,
        ctx: &ScanContext,
        on_conversation: &mut dyn FnMut(NormalizedConversation) -> Result<()>,
    ) -> Result<()> {
        for root in Self::source_roots(ctx) {
            if !root.path.exists() {
                continue;
            }
            for (dir_path, is_encrypted) in Self::find_conversation_dirs(&root.path) {
                if is_encrypted && self.encryption_key.is_none() {
                    tracing::debug!(
                        path = %dir_path.display(),
                        "chatgpt skipping encrypted directory because no decryption key is configured"
                    );
                    continue;
                }
                for path in Self::conversation_files(&dir_path) {
                    if !file_modified_since(&path, ctx.since_ts) {
                        continue;
                    }
                    match self.parse_conversation_file(&path, is_encrypted) {
                        Ok(Some(conversation)) => {
                            tracing::debug!(
                                path = %path.display(),
                                messages = conversation.messages.len(),
                                encrypted = is_encrypted,
                                "chatgpt extracted conversation"
                            );
                            on_conversation(conversation)?;
                        }
                        Ok(None) => {
                            tracing::debug!(
                                path = %path.display(),
                                "chatgpt no messages in conversation"
                            );
                        }
                        Err(error) => {
                            if is_encrypted {
                                tracing::warn!(
                                    path = %path.display(),
                                    error = %error,
                                    "chatgpt failed to decrypt/parse conversation"
                                );
                            } else {
                                tracing::warn!(
                                    path = %path.display(),
                                    error = %error,
                                    "chatgpt failed to parse conversation"
                                );
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

fn mapping_message_content(message: &Value) -> Option<String> {
    let content = message.get("content")?;
    if let Some(parts) = content.get("parts").and_then(|raw| raw.as_array()) {
        Some(
            parts
                .iter()
                .filter_map(|part| part.as_str())
                .collect::<Vec<_>>()
                .join("\n"),
        )
    } else {
        content
            .get("text")
            .and_then(|raw| raw.as_str())
            .map(String::from)
    }
}

fn update_time_bounds(started_at: &mut Option<i64>, ended_at: &mut Option<i64>, ts: Option<i64>) {
    if let Some(ts) = ts {
        *started_at = Some(started_at.map_or(ts, |current| current.min(ts)));
        *ended_at = Some(ended_at.map_or(ts, |current| current.max(ts)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::TempDir;

    #[test]
    fn chatgpt_connector_streams_each_unencrypted_file_via_callback() {
        let tmp = TempDir::new().unwrap();
        let conversations_dir = tmp.path().join("conversations-web-export");
        fs::create_dir_all(&conversations_dir).unwrap();
        fs::write(
            conversations_dir.join("b.json"),
            serde_json::to_vec(&json!({
                "id": "b",
                "title": "Second",
                "mapping": {
                    "node-b": {
                        "message": {
                            "author": {"role": "user"},
                            "create_time": 1.0,
                            "content": {"parts": ["second"]}
                        }
                    }
                }
            }))
            .unwrap(),
        )
        .unwrap();
        fs::write(
            conversations_dir.join("a.json"),
            serde_json::to_vec(&json!({
                "id": "a",
                "title": "First",
                "mapping": {
                    "node-a": {
                        "message": {
                            "author": {"role": "assistant"},
                            "create_time": 2.0,
                            "content": {"text": "first"}
                        }
                    }
                }
            }))
            .unwrap(),
        )
        .unwrap();

        let connector = ChatGptConnector::new();
        assert!(connector.supports_streaming_scan());
        let root = ScanRoot::local(conversations_dir.clone());
        let ctx = ScanContext::with_roots(conversations_dir, vec![root], None);
        let mut ids = Vec::new();
        connector
            .scan_with_callback(&ctx, &mut |conversation| {
                ids.push(conversation.external_id.unwrap_or_default());
                Ok(())
            })
            .unwrap();

        assert_eq!(ids, vec!["a".to_string(), "b".to_string()]);
    }
}
