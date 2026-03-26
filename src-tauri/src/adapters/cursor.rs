use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::path::Path;

use anyhow::{anyhow, Result};
use glob::glob;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use crate::sync::normalize::{ConversationImport, ImportedConversation, MessageImport};

pub struct CursorAdapter;

#[derive(Debug, Deserialize)]
struct FixtureGlobalStorage {
    rows: Vec<FixtureGlobalStorageRow>,
}

#[derive(Debug, Deserialize)]
struct FixtureGlobalStorageRow {
    key: String,
    value: Value,
}

#[derive(Debug, Deserialize, Serialize)]
struct WorkspaceComposerData {
    #[serde(rename = "allComposers")]
    all_composers: Vec<WorkspaceComposerHead>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct WorkspaceComposerHead {
    #[serde(rename = "composerId")]
    composer_id: String,
    name: Option<String>,
    subtitle: Option<String>,
    #[serde(rename = "createdAt")]
    created_at: Option<i64>,
    #[serde(rename = "lastUpdatedAt")]
    last_updated_at: Option<i64>,
    #[serde(rename = "createdOnBranch")]
    created_on_branch: Option<String>,
    #[serde(rename = "filesChangedCount")]
    files_changed_count: Option<i64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct GenerationEntry {
    #[serde(rename = "unixMs")]
    unix_ms: i64,
    #[serde(rename = "generationUUID")]
    generation_uuid: String,
    #[serde(rename = "composerId")]
    composer_id: Option<String>,
    #[serde(rename = "textDescription")]
    text_description: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
struct PromptEntry {
    text: Option<String>,
    #[serde(rename = "commandType")]
    command_type: Option<i64>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
struct WorkspaceFile {
    folder: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
struct WorkspacePersistentData {
    #[serde(rename = "cachedSelectedGitState")]
    cached_selected_git_state: Option<CachedSelectedGitState>,
    #[serde(rename = "cachedSelectedRemote")]
    cached_selected_remote: Option<CachedSelectedRemote>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
struct CachedSelectedGitState {
    #[serde(rename = "ref")]
    git_ref: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
struct CachedSelectedRemote {
    url: Option<String>,
    #[serde(rename = "rootUri")]
    root_uri: Option<WorkspaceUri>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
struct WorkspaceUri {
    path: Option<String>,
    external: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct GlobalComposerData {
    #[serde(rename = "composerId")]
    composer_id: String,
    text: Option<String>,
    status: Option<String>,
    #[serde(default, rename = "fullConversationHeadersOnly")]
    full_conversation_headers_only: Vec<BubbleHeader>,
    #[serde(rename = "createdAt")]
    created_at: Option<i64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct BubbleHeader {
    #[serde(rename = "bubbleId")]
    bubble_id: String,
    #[serde(rename = "type")]
    bubble_type: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct GlobalBubbleData {
    #[serde(rename = "type")]
    bubble_type: i64,
    #[serde(rename = "bubbleId")]
    bubble_id: String,
    #[serde(rename = "createdAt")]
    created_at: Option<String>,
    text: Option<String>,
    #[serde(rename = "richText")]
    rich_text: Option<String>,
    #[serde(rename = "capabilityType")]
    capability_type: Option<i64>,
    thinking: Option<ThinkingBlock>,
    #[serde(rename = "toolFormerData")]
    tool_former_data: Option<ToolFormerData>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
struct ThinkingBlock {
    text: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
struct ToolFormerData {
    name: Option<String>,
    params: Option<String>,
    result: Option<String>,
}

#[derive(Debug, Clone, Default)]
struct WorkspaceContext {
    workspace_path: Option<String>,
    git_branch: Option<String>,
    git_remote: Option<String>,
}

#[derive(Debug, Clone)]
struct WorkspaceComposerEntry {
    composer: WorkspaceComposerHead,
    workspace_context: WorkspaceContext,
    generation_count: usize,
    prompt_count: usize,
}

#[derive(Debug, Clone)]
struct GlobalComposerRecord {
    composer: GlobalComposerData,
    raw_value: Value,
    bubbles: HashMap<String, GlobalBubbleRecord>,
}

#[derive(Debug, Clone)]
struct GlobalBubbleRecord {
    bubble: GlobalBubbleData,
    raw_value: Value,
}

#[derive(Debug, Default)]
struct RecoveredConversation {
    messages: Vec<MessageImport>,
    first_user_text: Option<String>,
    first_message_at: Option<i64>,
    last_message_at: Option<i64>,
    bubble_header_count: usize,
    recovered_bubble_count: usize,
    missing_bubble_count: usize,
    ignored_thinking_block_count: usize,
}

impl CursorAdapter {
    pub fn parse_fixture_dir<P: AsRef<Path>>(fixture_dir: P) -> Result<Vec<ImportedConversation>> {
        let fixture_dir = fixture_dir.as_ref();
        let global_storage = load_global_storage_from_fixture(&fixture_dir.join("global_storage.json"))?;
        let workspace_index =
            load_workspace_index_from_fixture(&fixture_dir.join("workspace_storage"))?;

        Ok(build_imports(global_storage, workspace_index))
    }

    pub fn import_default_sources() -> Result<Vec<ImportedConversation>> {
        let workspace_index = load_workspace_index_from_default_sources()?;
        let home = env::var("HOME").map_err(|_| anyhow!("HOME is not set"))?;
        let global_storage_path =
            Path::new(&home).join("Library/Application Support/Cursor/User/globalStorage/state.vscdb");

        let global_storage = if global_storage_path.exists() {
            match load_global_storage_from_db(&global_storage_path) {
                Ok(storage) => storage,
                Err(_) => Vec::new(),
            }
        } else {
            Vec::new()
        };

        Ok(build_imports(global_storage, workspace_index))
    }
}

fn build_imports(
    global_storage: Vec<GlobalComposerRecord>,
    workspace_index: HashMap<String, WorkspaceComposerEntry>,
) -> Vec<ImportedConversation> {
    let mut imports = Vec::new();
    let mut seen_workspace_ids = HashSet::new();

    for global in global_storage {
        let composer_id = global.composer.composer_id.clone();
        let workspace_entry = workspace_index.get(&composer_id);
        let recovered = recover_conversation(&global);

        if !recovered.messages.is_empty() {
            imports.push(build_partial_import(&global, workspace_entry, recovered));
            if workspace_entry.is_some() {
                seen_workspace_ids.insert(composer_id);
            }
            continue;
        }

        if let Some(entry) = workspace_entry {
            imports.push(build_metadata_only_import(
                entry,
                Some(&global),
                "no_recoverable_bubbles",
                Some(&recovered),
            ));
            seen_workspace_ids.insert(composer_id);
        }
    }

    for (composer_id, workspace_entry) in workspace_index {
        if seen_workspace_ids.contains(&composer_id) {
            continue;
        }

        imports.push(build_metadata_only_import(
            &workspace_entry,
            None,
            "workspace_only",
            None,
        ));
    }

    imports.sort_by(|left, right| {
        right
            .conversation
            .updated_at
            .cmp(&left.conversation.updated_at)
    });

    imports
}

fn recover_conversation(global: &GlobalComposerRecord) -> RecoveredConversation {
    let mut recovered = RecoveredConversation {
        bubble_header_count: global.composer.full_conversation_headers_only.len(),
        ..RecoveredConversation::default()
    };

    for header in &global.composer.full_conversation_headers_only {
        let Some(bubble_record) = global.bubbles.get(&header.bubble_id) else {
            recovered.missing_bubble_count += 1;
            continue;
        };

        recovered.recovered_bubble_count += 1;

        let Some(created_at) = bubble_record
            .bubble
            .created_at
            .as_deref()
            .and_then(|value| parse_rfc3339_millis(value).ok())
        else {
            continue;
        };

        if recovered.first_message_at.is_none() {
            recovered.first_message_at = Some(created_at);
        }

        if bubble_record
            .bubble
            .thinking
            .as_ref()
            .and_then(|thinking| normalize_string(thinking.text.clone()))
            .is_some()
        {
            recovered.ignored_thinking_block_count += 1;
        }

        if header.bubble_type == 1 {
            if let Some(text) = extract_user_text(&bubble_record.bubble) {
                if recovered.first_user_text.is_none() {
                    recovered.first_user_text = Some(text.clone());
                }
                recovered.messages.push(MessageImport {
                    source_message_id: bubble_record.bubble.bubble_id.clone(),
                    role: "user".into(),
                    message_type: "user".into(),
                    content_text: text,
                    tool_name: None,
                    created_at,
                    raw_payload_json: bubble_record.raw_value.to_string(),
                });
                recovered.last_message_at = Some(created_at);
            }

            continue;
        }

        if let Some(text) = normalize_string(bubble_record.bubble.text.clone()) {
            recovered.messages.push(MessageImport {
                source_message_id: bubble_record.bubble.bubble_id.clone(),
                role: "assistant".into(),
                message_type: "assistant".into(),
                content_text: text,
                tool_name: None,
                created_at,
                raw_payload_json: bubble_record.raw_value.to_string(),
            });
            recovered.last_message_at = Some(created_at);
        }

        if let Some(tool_data) = bubble_record.bubble.tool_former_data.as_ref() {
            let tool_name = normalize_string(tool_data.name.clone());
            let tool_call_text = format_tool_call_content(tool_data.params.as_deref(), tool_name.clone());
            if !tool_call_text.is_empty() {
                recovered.messages.push(MessageImport {
                    source_message_id: format!("{}:tool_call", bubble_record.bubble.bubble_id),
                    role: "assistant".into(),
                    message_type: "tool_call".into(),
                    content_text: tool_call_text,
                    tool_name: tool_name.clone(),
                    created_at,
                    raw_payload_json: bubble_record.raw_value.to_string(),
                });
                recovered.last_message_at = Some(created_at);
            }

            let tool_result_text = format_tool_result_content(tool_data.result.as_deref());
            if !tool_result_text.is_empty() {
                recovered.messages.push(MessageImport {
                    source_message_id: format!("{}:tool_result", bubble_record.bubble.bubble_id),
                    role: "tool".into(),
                    message_type: "tool_result".into(),
                    content_text: tool_result_text,
                    tool_name,
                    created_at,
                    raw_payload_json: bubble_record.raw_value.to_string(),
                });
                recovered.last_message_at = Some(created_at);
            }
        }
    }

    recovered
}

fn build_partial_import(
    global: &GlobalComposerRecord,
    workspace_entry: Option<&WorkspaceComposerEntry>,
    recovered: RecoveredConversation,
) -> ImportedConversation {
    let workspace_path = workspace_entry
        .and_then(|entry| entry.workspace_context.workspace_path.clone());
    let git_branch = workspace_entry
        .and_then(|entry| entry.workspace_context.git_branch.clone())
        .or_else(|| {
            workspace_entry.and_then(|entry| entry.composer.created_on_branch.clone())
        });
    let git_remote = workspace_entry
        .and_then(|entry| entry.workspace_context.git_remote.clone());
    let title = workspace_entry
        .and_then(|entry| normalize_string(entry.composer.name.clone()))
        .or_else(|| normalize_string(global.composer.text.clone()))
        .or_else(|| {
            recovered
                .first_user_text
                .as_deref()
                .map(truncate_title)
                .map(str::to_owned)
        })
        .unwrap_or_else(|| "Untitled Cursor Composer".into());
    let subtitle = workspace_entry.and_then(|entry| entry.composer.subtitle.clone());
    let created_at = recovered
        .first_message_at
        .or(workspace_entry.and_then(|entry| entry.composer.created_at))
        .or(global.composer.created_at)
        .unwrap_or(0);
    let updated_at = recovered
        .last_message_at
        .or(workspace_entry.and_then(|entry| entry.composer.last_updated_at))
        .or(workspace_entry.and_then(|entry| entry.composer.created_at))
        .or(global.composer.created_at)
        .unwrap_or(created_at);

    ImportedConversation {
        conversation: ConversationImport {
            source_app: "cursor".into(),
            source_conversation_id: global.composer.composer_id.clone(),
            title,
            subtitle,
            created_at,
            updated_at,
            sync_strength: "partial".into(),
            raw_metadata_json: json!({
                "workspace_path": workspace_path,
                "git_branch": git_branch,
                "git_remote": git_remote,
                "global_composer": global.raw_value,
                "workspace_composer": workspace_entry.map(|entry| json!(entry.composer)),
                "bubble_header_count": recovered.bubble_header_count,
                "recovered_bubble_count": recovered.recovered_bubble_count,
                "missing_bubble_count": recovered.missing_bubble_count,
                "recovered_message_count": recovered.messages.len(),
                "ignored_thinking_block_count": recovered.ignored_thinking_block_count,
            })
            .to_string(),
        },
        messages: recovered.messages,
    }
}

fn build_metadata_only_import(
    workspace_entry: &WorkspaceComposerEntry,
    global: Option<&GlobalComposerRecord>,
    fallback_reason: &str,
    recovered: Option<&RecoveredConversation>,
) -> ImportedConversation {
    let title = normalize_string(workspace_entry.composer.name.clone())
        .or_else(|| global.and_then(|item| normalize_string(item.composer.text.clone())))
        .unwrap_or_else(|| "Untitled Cursor Composer".into());
    let created_at = workspace_entry
        .composer
        .created_at
        .or_else(|| global.and_then(|item| item.composer.created_at))
        .unwrap_or(0);
    let updated_at = workspace_entry
        .composer
        .last_updated_at
        .or(workspace_entry.composer.created_at)
        .or_else(|| recovered.and_then(|item| item.last_message_at))
        .or_else(|| global.and_then(|item| item.composer.created_at))
        .unwrap_or(created_at);

    ImportedConversation {
        conversation: ConversationImport {
            source_app: "cursor".into(),
            source_conversation_id: workspace_entry.composer.composer_id.clone(),
            title,
            subtitle: workspace_entry.composer.subtitle.clone(),
            created_at,
            updated_at,
            sync_strength: "metadata_only".into(),
            raw_metadata_json: json!({
                "workspace_path": workspace_entry.workspace_context.workspace_path,
                "git_branch": workspace_entry
                    .workspace_context
                    .git_branch
                    .clone()
                    .or_else(|| workspace_entry.composer.created_on_branch.clone()),
                "git_remote": workspace_entry.workspace_context.git_remote,
                "workspace_composer": workspace_entry.composer,
                "global_composer": global.map(|item| &item.raw_value),
                "generation_count": workspace_entry.generation_count,
                "prompt_count": workspace_entry.prompt_count,
                "fallback_reason": fallback_reason,
                "bubble_header_count": recovered.map(|item| item.bubble_header_count),
                "recovered_bubble_count": recovered.map(|item| item.recovered_bubble_count),
                "missing_bubble_count": recovered.map(|item| item.missing_bubble_count),
                "ignored_thinking_block_count": recovered.map(|item| item.ignored_thinking_block_count),
            })
            .to_string(),
        },
        messages: Vec::new(),
    }
}

fn load_global_storage_from_fixture(path: &Path) -> Result<Vec<GlobalComposerRecord>> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let payload: FixtureGlobalStorage = serde_json::from_str(&fs::read_to_string(path)?)?;
    let mut composer_rows: Vec<(String, Value)> = Vec::new();
    let mut bubble_rows: HashMap<String, Vec<(String, Value)>> = HashMap::new();

    for row in payload.rows {
        if let Some(composer_id) = row.key.strip_prefix("composerData:") {
            composer_rows.push((composer_id.to_string(), row.value));
            continue;
        }

        if let Some((composer_id, _bubble_id)) = split_bubble_key(&row.key) {
            bubble_rows
                .entry(composer_id)
                .or_default()
                .push((row.key, row.value));
        }
    }

    build_global_storage_from_rows(composer_rows, bubble_rows)
}

fn load_global_storage_from_db(path: &Path) -> Result<Vec<GlobalComposerRecord>> {
    let connection = Connection::open(path)?;
    let mut composer_values = Vec::new();
    let mut bubble_rows: HashMap<String, Vec<(String, Value)>> = HashMap::new();
    for (key, raw_value) in collect_text_rows(
        &connection,
        "SELECT key, CAST(value AS TEXT) FROM cursorDiskKV WHERE key LIKE 'composerData:%'",
        [],
    )? {
        let Some(composer_id) = key.strip_prefix("composerData:") else {
            continue;
        };
        let value: Value = match serde_json::from_str(&raw_value) {
            Ok(value) => value,
            Err(_) => continue,
        };
        composer_values.push((composer_id.to_string(), value));

        let pattern = format!("bubbleId:{composer_id}:%");
        let mut bubbles = Vec::new();
        for (bubble_key, bubble_value) in collect_text_rows(
            &connection,
            "SELECT key, CAST(value AS TEXT) FROM cursorDiskKV WHERE key LIKE ?1",
            [pattern],
        )? {
            let value: Value = match serde_json::from_str(&bubble_value) {
                Ok(value) => value,
                Err(_) => continue,
            };
            bubbles.push((bubble_key, value));
        }
        bubble_rows.insert(composer_id.to_string(), bubbles);
    }

    build_global_storage_from_rows(composer_values, bubble_rows)
}

fn collect_text_rows<P>(connection: &Connection, sql: &str, params: P) -> Result<Vec<(String, String)>>
where
    P: rusqlite::Params,
{
    let mut statement = connection.prepare(sql)?;
    let mut rows = statement.query(params)?;
    let mut results = Vec::new();

    while let Some(row) = rows.next()? {
        let key: String = row.get(0)?;
        let value: Option<String> = row.get(1)?;
        let Some(value) = value else {
            continue;
        };
        results.push((key, value));
    }

    Ok(results)
}

fn build_global_storage_from_rows(
    composer_rows: Vec<(String, Value)>,
    bubble_rows: HashMap<String, Vec<(String, Value)>>,
) -> Result<Vec<GlobalComposerRecord>> {
    let mut composers = Vec::new();

    for (composer_id, raw_value) in composer_rows {
        let composer: GlobalComposerData = match serde_json::from_value(raw_value.clone()) {
            Ok(value) => value,
            Err(_) => continue,
        };
        if composer.composer_id != composer_id {
            continue;
        }

        let bubbles = bubble_rows
            .get(&composer_id)
            .into_iter()
            .flatten()
            .filter_map(|(key, raw_bubble)| {
                let Some((_bubble_composer_id, bubble_id)) = split_bubble_key(key) else {
                    return None;
                };
                let bubble: GlobalBubbleData = serde_json::from_value(raw_bubble.clone()).ok()?;
                Some((
                    bubble_id,
                    GlobalBubbleRecord {
                        bubble,
                        raw_value: raw_bubble.clone(),
                    },
                ))
            })
            .collect::<HashMap<_, _>>();

        composers.push(GlobalComposerRecord {
            composer,
            raw_value,
            bubbles,
        });
    }

    Ok(composers)
}

fn load_workspace_index_from_fixture(path: &Path) -> Result<HashMap<String, WorkspaceComposerEntry>> {
    if !path.exists() {
        return Ok(HashMap::new());
    }

    let mut index = HashMap::new();

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }

        let snapshot_dir = entry.path();
        for composer_entry in load_workspace_snapshot_from_dir(&snapshot_dir)? {
            insert_workspace_entry(&mut index, composer_entry);
        }
    }

    Ok(index)
}

fn load_workspace_index_from_default_sources() -> Result<HashMap<String, WorkspaceComposerEntry>> {
    let home = env::var("HOME").map_err(|_| anyhow!("HOME is not set"))?;
    let pattern = format!(
        "{home}/Library/Application Support/Cursor/User/workspaceStorage/**/state.vscdb"
    );
    let mut index = HashMap::new();

    for path in glob(&pattern)? {
        let path = path?;
        let entries = match load_workspace_snapshot_from_db(&path) {
            Ok(entries) => entries,
            Err(error) if error.to_string().contains("Query returned no rows") => continue,
            Err(error) => return Err(error),
        };

        for entry in entries {
            insert_workspace_entry(&mut index, entry);
        }
    }

    Ok(index)
}

fn load_workspace_snapshot_from_dir(path: &Path) -> Result<Vec<WorkspaceComposerEntry>> {
    let composer_data: WorkspaceComposerData = serde_json::from_str(&fs::read_to_string(
        path.join("composer_data.json"),
    )?)?;
    let generations =
        read_optional_json_file::<Vec<GenerationEntry>>(&path.join("generations.json"))?
            .unwrap_or_default();
    let prompts = read_optional_json_file::<Vec<PromptEntry>>(&path.join("prompts.json"))?
        .unwrap_or_default();
    let workspace_file = read_optional_json_file::<WorkspaceFile>(&path.join("workspace.json"))?;
    let workspace_persistent_data = read_optional_json_file::<WorkspacePersistentData>(
        &path.join("workspace_persistent_data.json"),
    )?;
    let workspace_context =
        WorkspaceContext::from_sources(workspace_file, workspace_persistent_data);

    Ok(composer_data
        .all_composers
        .into_iter()
        .map(|composer| WorkspaceComposerEntry {
            composer,
            workspace_context: workspace_context.clone(),
            generation_count: generations.len(),
            prompt_count: prompts.len(),
        })
        .collect())
}

fn load_workspace_snapshot_from_db(path: &Path) -> Result<Vec<WorkspaceComposerEntry>> {
    let connection = Connection::open(path)?;
    let composer_json: String = connection.query_row(
        "SELECT value FROM ItemTable WHERE key = 'composer.composerData'",
        [],
        |row| row.get(0),
    )?;
    let composer_data: WorkspaceComposerData = serde_json::from_str(&composer_json)?;
    let generations =
        query_optional_json::<Vec<GenerationEntry>>(&connection, "aiService.generations")?
            .unwrap_or_default();
    let prompts = query_optional_json::<Vec<PromptEntry>>(&connection, "aiService.prompts")?
        .unwrap_or_default();
    let workspace_file = read_optional_json_file::<WorkspaceFile>(
        &path.parent().unwrap_or_else(|| Path::new(".")).join("workspace.json"),
    )?;
    let workspace_persistent_data = query_optional_json::<WorkspacePersistentData>(
        &connection,
        "workbench.backgroundComposer.workspacePersistentData",
    )?;
    let workspace_context =
        WorkspaceContext::from_sources(workspace_file, workspace_persistent_data);

    Ok(composer_data
        .all_composers
        .into_iter()
        .map(|composer| WorkspaceComposerEntry {
            composer,
            workspace_context: workspace_context.clone(),
            generation_count: generations.len(),
            prompt_count: prompts.len(),
        })
        .collect())
}

impl WorkspaceContext {
    fn from_sources(
        workspace_file: Option<WorkspaceFile>,
        workspace_persistent_data: Option<WorkspacePersistentData>,
    ) -> Self {
        let workspace_path = workspace_file
            .and_then(|workspace| workspace.folder)
            .as_deref()
            .and_then(parse_file_uri)
            .or_else(|| {
                workspace_persistent_data
                    .as_ref()
                    .and_then(|data| data.cached_selected_remote.as_ref())
                    .and_then(|remote| {
                        remote
                            .root_uri
                            .as_ref()
                            .and_then(|uri| normalize_string(uri.path.clone()))
                            .or_else(|| {
                                uri_external_path(
                                    remote.root_uri.as_ref().and_then(|uri| uri.external.as_deref()),
                                )
                            })
                    })
            });
        let git_branch = workspace_persistent_data
            .as_ref()
            .and_then(|data| data.cached_selected_git_state.as_ref())
            .and_then(|state| normalize_string(state.git_ref.clone()));
        let git_remote = workspace_persistent_data
            .and_then(|data| data.cached_selected_remote)
            .and_then(|remote| normalize_string(remote.url));

        Self {
            workspace_path,
            git_branch,
            git_remote,
        }
    }
}

fn insert_workspace_entry(
    index: &mut HashMap<String, WorkspaceComposerEntry>,
    candidate: WorkspaceComposerEntry,
) {
    let candidate_score = workspace_entry_score(&candidate);
    index
        .entry(candidate.composer.composer_id.clone())
        .and_modify(|current| {
            if candidate_score > workspace_entry_score(current) {
                *current = candidate.clone();
            }
        })
        .or_insert(candidate);
}

fn workspace_entry_score(entry: &WorkspaceComposerEntry) -> (i64, i64, i64) {
    (
        entry
            .composer
            .last_updated_at
            .or(entry.composer.created_at)
            .unwrap_or(0),
        entry.composer.name.is_some() as i64,
        entry.composer.subtitle.is_some() as i64,
    )
}

fn extract_user_text(bubble: &GlobalBubbleData) -> Option<String> {
    normalize_string(bubble.text.clone())
        .or_else(|| bubble.rich_text.as_deref().and_then(extract_rich_text))
}

fn extract_rich_text(value: &str) -> Option<String> {
    let parsed: Value = serde_json::from_str(value).ok()?;
    let mut parts = Vec::new();
    collect_rich_text_segments(&parsed, &mut parts);
    normalize_string(Some(parts.join("")))
}

fn collect_rich_text_segments(node: &Value, parts: &mut Vec<String>) {
    if let Some(text) = node.get("text").and_then(Value::as_str) {
        parts.push(text.to_string());
    } else if node.get("type").and_then(Value::as_str) == Some("mention") {
        if let Some(name) = node.get("mentionName").and_then(Value::as_str) {
            parts.push(name.to_string());
        }
    }

    if let Some(children) = node.get("children").and_then(Value::as_array) {
        for child in children {
            collect_rich_text_segments(child, parts);
        }
    }
}

fn format_tool_call_content(params: Option<&str>, tool_name: Option<String>) -> String {
    let Some(params) = params else {
        return tool_name.unwrap_or_default();
    };

    if let Ok(value) = serde_json::from_str::<Value>(params) {
        if let Some(command) = value.get("command").and_then(Value::as_str) {
            return command.to_string();
        }

        return serde_json::to_string_pretty(&value).unwrap_or_else(|_| params.to_string());
    }

    params.to_string()
}

fn format_tool_result_content(result: Option<&str>) -> String {
    let Some(result) = result else {
        return String::new();
    };

    if let Ok(value) = serde_json::from_str::<Value>(result) {
        if let Some(output) = value.get("output").and_then(Value::as_str) {
            return output.to_string();
        }

        return serde_json::to_string_pretty(&value).unwrap_or_else(|_| result.to_string());
    }

    result.to_string()
}

fn split_bubble_key(key: &str) -> Option<(String, String)> {
    let remainder = key.strip_prefix("bubbleId:")?;
    let mut parts = remainder.splitn(2, ':');
    let composer_id = parts.next()?.to_string();
    let bubble_id = parts.next()?.to_string();
    Some((composer_id, bubble_id))
}

fn read_optional_json_file<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<Option<T>> {
    if !path.exists() {
        return Ok(None);
    }

    let contents = fs::read_to_string(path)?;
    Ok(Some(serde_json::from_str(&contents)?))
}

fn query_optional_json<T: for<'de> Deserialize<'de>>(
    connection: &Connection,
    key: &str,
) -> Result<Option<T>> {
    let value: Option<String> = connection
        .query_row(
            "SELECT value FROM ItemTable WHERE key = ?1",
            [key],
            |row| row.get(0),
        )
        .ok();

    value
        .as_deref()
        .map(serde_json::from_str)
        .transpose()
        .map_err(Into::into)
}

fn normalize_string(value: Option<String>) -> Option<String> {
    value
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
}

fn parse_file_uri(value: &str) -> Option<String> {
    let path = value.strip_prefix("file://")?;
    Some(percent_decode(path))
}

fn uri_external_path(value: Option<&str>) -> Option<String> {
    value.and_then(parse_file_uri)
}

fn percent_decode(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut output = Vec::with_capacity(value.len());
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            let maybe_hex = std::str::from_utf8(&bytes[index + 1..index + 3]).ok();
            if let Some(hex) = maybe_hex.and_then(|item| u8::from_str_radix(item, 16).ok()) {
                output.push(hex);
                index += 3;
                continue;
            }
        }

        output.push(bytes[index]);
        index += 1;
    }

    String::from_utf8_lossy(&output).into_owned()
}

fn parse_rfc3339_millis(value: &str) -> Result<i64> {
    let parsed = OffsetDateTime::parse(value, &Rfc3339)?;
    Ok(parsed.unix_timestamp_nanos() as i64 / 1_000_000)
}

fn truncate_title(input: &str) -> &str {
    const MAX_CHARS: usize = 48;
    if input.chars().count() <= MAX_CHARS {
        return input;
    }

    let end = input
        .char_indices()
        .nth(MAX_CHARS)
        .map(|(index, _)| index)
        .unwrap_or(input.len());

    &input[..end]
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;
    use tempfile::NamedTempFile;

    use super::load_global_storage_from_db;

    #[test]
    fn load_global_storage_from_db_skips_null_composer_rows() {
        let file = NamedTempFile::new().unwrap();
        let connection = Connection::open(file.path()).unwrap();
        connection
            .execute("CREATE TABLE cursorDiskKV (key TEXT, value BLOB)", [])
            .unwrap();
        connection
            .execute(
                "INSERT INTO cursorDiskKV (key, value) VALUES (?1, NULL)",
                ["composerData:null-row"],
            )
            .unwrap();
        connection
            .execute(
                "INSERT INTO cursorDiskKV (key, value) VALUES (?1, ?2)",
                [
                    "composerData:valid-row",
                    r#"{"composerId":"valid-row","status":"completed","fullConversationHeadersOnly":[{"bubbleId":"bubble-1","type":1}]}"#,
                ],
            )
            .unwrap();
        connection
            .execute(
                "INSERT INTO cursorDiskKV (key, value) VALUES (?1, ?2)",
                [
                    "bubbleId:valid-row:bubble-1",
                    r#"{"type":1,"bubbleId":"bubble-1","createdAt":"2026-03-20T09:17:52.323Z","text":"hello"}"#,
                ],
            )
            .unwrap();
        drop(connection);

        let rows = load_global_storage_from_db(file.path()).unwrap();

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].composer.composer_id, "valid-row");
        assert_eq!(rows[0].bubbles.len(), 1);
    }

    #[test]
    fn load_global_storage_from_db_processes_all_composer_rows() {
        let file = NamedTempFile::new().unwrap();
        let connection = Connection::open(file.path()).unwrap();
        connection
            .execute("CREATE TABLE cursorDiskKV (key TEXT, value BLOB)", [])
            .unwrap();
        connection
            .execute(
                "INSERT INTO cursorDiskKV (key, value) VALUES (?1, ?2)",
                [
                    "composerData:first-row",
                    r#"{"composerId":"first-row","status":"completed","fullConversationHeadersOnly":[]}"#,
                ],
            )
            .unwrap();
        connection
            .execute(
                "INSERT INTO cursorDiskKV (key, value) VALUES (?1, ?2)",
                [
                    "composerData:second-row",
                    r#"{"composerId":"second-row","status":"completed","fullConversationHeadersOnly":[{"bubbleId":"bubble-1","type":1}]}"#,
                ],
            )
            .unwrap();
        connection
            .execute(
                "INSERT INTO cursorDiskKV (key, value) VALUES (?1, ?2)",
                [
                    "bubbleId:second-row:bubble-1",
                    r#"{"type":1,"bubbleId":"bubble-1","createdAt":"2026-03-20T09:17:52.323Z","text":"hello"}"#,
                ],
            )
            .unwrap();
        drop(connection);

        let rows = load_global_storage_from_db(file.path()).unwrap();

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].composer.composer_id, "first-row");
        assert_eq!(rows[1].composer.composer_id, "second-row");
        assert_eq!(rows[1].bubbles.len(), 1);
    }
}
