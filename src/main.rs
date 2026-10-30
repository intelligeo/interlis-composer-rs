mod model;
mod toolchain_adapter;

use axum::{
    extract::State,
    http::{header, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use model::{Association, CatalogInstance, CatalogValidationPreset, Class, Domain, InterlisModel, InterlisProject, ProseSection, Topic, from_interlis};
use toolchain_adapter::{ToolCommand, ToolExecError, ToolIssue, ToolRequest, ToolchainConfig};
use std::collections::{HashMap, HashSet};
use std::{
    env,
    fs,
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
    trace::TraceLayer,
};
use tracing::info;
use tracing_appender::non_blocking::{NonBlocking, WorkerGuard};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone)]
struct AppState {
    defaults: Arc<Defaults>,
    tooling: Arc<ToolchainConfig>,
    metrics: Arc<Mutex<OpsMetrics>>,
}

#[derive(Clone)]
struct Defaults {
    model_name: String,
    language: String,
}

#[derive(serde::Serialize)]
struct HealthResponse {
    status: &'static str,
}

#[derive(serde::Serialize)]
struct ValidationIssue {
    level: &'static str,
    code: &'static str,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    node_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    line: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    column: Option<usize>,
}

#[derive(serde::Serialize)]
struct ValidationResponse {
    valid: bool,
    issues: Vec<ValidationIssue>,
}

#[derive(serde::Serialize)]
struct ExportResponse {
    file_name: String,
    interlis: String,
    issues: Vec<ValidationIssue>,
}

#[derive(serde::Deserialize)]
struct NewProjectRequest {
    project_name: Option<String>,
    model_name: Option<String>,
    language: Option<String>,
}

#[derive(serde::Serialize)]
struct ProjectImportResponse {
    valid: bool,
    project: InterlisProject,
    issues: Vec<ValidationIssue>,
}

#[derive(serde::Serialize)]
struct ProjectExportResponse {
    file_name: String,
    project_json: String,
    issues: Vec<ValidationIssue>,
}

#[derive(serde::Deserialize)]
struct ImportIliRequest {
    interlis: String,
    current_model: Option<InterlisModel>,
    on_conflict: Option<String>,
}

#[derive(serde::Serialize)]
struct ImportIliResponse {
    valid: bool,
    model: InterlisModel,
    issues: Vec<ValidationIssue>,
}

#[derive(serde::Deserialize)]
struct TransferRequest {
    #[allow(dead_code)]
    file_name: Option<String>,
    #[allow(dead_code)]
    content: String,
    #[allow(dead_code)]
    model: Option<InterlisModel>,
}

#[derive(serde::Serialize)]
struct TransferResponse {
    valid: bool,
    issues: Vec<ValidationIssue>,
}

#[derive(serde::Serialize)]
struct DocsExportResponse {
    file_name: String,
    markdown: String,
    issues: Vec<ValidationIssue>,
}

#[derive(serde::Serialize)]
struct CollabStatusResponse {
    repository_detected: bool,
    branch: Option<String>,
    head_commit: Option<String>,
    capabilities: Vec<String>,
    notes: Vec<String>,
}

#[derive(serde::Deserialize)]
struct CommitDraftRequest {
    model: InterlisModel,
    focus: Option<String>,
}

#[derive(serde::Serialize)]
struct CommitDraftResponse {
    title: String,
    body: String,
    stats: HashMap<String, usize>,
}

#[derive(Debug, Clone, serde::Serialize)]
struct GitRefEntry {
    name: String,
    sha: String,
}

#[derive(serde::Serialize)]
struct GitRefsResponse {
    repository_detected: bool,
    branches: Vec<GitRefEntry>,
    tags: Vec<GitRefEntry>,
    notes: Vec<String>,
}

#[derive(serde::Deserialize)]
struct ModelDiffRequest {
    current_model: InterlisModel,
    bundle_model: InterlisModel,
}

#[derive(serde::Serialize)]
struct ModelDiffResponse {
    same: bool,
    summary: HashMap<String, i64>,
    details: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
struct OpsMetrics {
    started_unix_s: u64,
    total_requests: u64,
    failed_requests: u64,
    by_endpoint: HashMap<String, u64>,
}

impl OpsMetrics {
    fn new() -> Self {
        Self {
            started_unix_s: now_unix_s(),
            total_requests: 0,
            failed_requests: 0,
            by_endpoint: HashMap::new(),
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let env_filter = std::env::var("RUST_LOG")
        .unwrap_or_else(|_| "interlis_composer_rs=info,tower_http=info".to_string());

    let mut _file_log_guard: Option<WorkerGuard> = None;
    let file_logger = match setup_file_logger() {
        Ok((writer, guard, path)) => {
            _file_log_guard = Some(guard);
            Some((writer, path))
        }
        Err(err) => {
            eprintln!("[logging] file logger disabled: {}", err);
            None
        }
    };

    if let Some((file_writer, log_path)) = file_logger {
        let file_layer = fmt::layer()
            .with_ansi(false)
            .with_writer(file_writer)
            .with_target(true)
            .with_thread_ids(true)
            .with_thread_names(true);

        tracing_subscriber::registry()
            .with(tracing_subscriber::EnvFilter::new(env_filter))
            .with(fmt::layer().with_target(true))
            .with(file_layer)
            .init();
        info!(log_file = %log_path, "file logging enabled");
    } else {
        tracing_subscriber::registry()
            .with(tracing_subscriber::EnvFilter::new(env_filter))
            .with(fmt::layer().with_target(true))
            .init();
    }

    let state = AppState {
        defaults: Arc::new(Defaults {
            model_name: env::var("DEFAULT_MODEL_NAME").unwrap_or_else(|_| "ModelName".to_string()),
            language: env::var("DEFAULT_MODEL_LANG").unwrap_or_else(|_| "de".to_string()),
        }),
        tooling: Arc::new(toolchain_adapter::load_config()),
        metrics: Arc::new(Mutex::new(OpsMetrics::new())),
    };

    let app = Router::new()
        .route("/health", get(health))
        .route("/api/ops/metrics", get(get_ops_metrics))
        .route("/api/project/new", post(new_project))
        .route("/api/project/import/imcp", post(import_imcp))
        .route("/api/project/export/imcp", post(export_imcp))
        .route("/api/collab/status", get(collab_status))
        .route("/api/collab/git-refs", get(collab_git_refs))
        .route("/api/collab/commit-draft", post(generate_commit_draft))
        .route("/api/collab/model-diff", post(compare_bundle_with_model))
        .route("/api/model/import/ili", post(import_ili))
        .route("/api/docs/export/markdown", post(export_docs_markdown))
        .route("/api/instance/import/xtf", post(import_xtf))
        .route("/api/instance/export/xtf", post(export_xtf))
        .route("/api/model/validate", post(validate_model))
        .route("/api/model/export/ili", post(export_ili))
        .nest_service("/", ServeDir::new("static").append_index_html_on_directories(true))
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(state);

    let port = env::var("PORT")
        .ok()
        .and_then(|v| v.parse::<u16>().ok())
        .unwrap_or(8080);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!(%addr, "starting server");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn health(State(state): State<AppState>) -> Json<HealthResponse> {
    track_api_request(&state, "/health", true);
    Json(HealthResponse { status: "ok" })
}

async fn get_ops_metrics(State(state): State<AppState>) -> Json<OpsMetrics> {
    track_api_request(&state, "/api/ops/metrics", true);
    match state.metrics.lock() {
        Ok(guard) => Json(guard.clone()),
        Err(_) => Json(OpsMetrics::new()),
    }
}

async fn collab_status(State(state): State<AppState>) -> Json<CollabStatusResponse> {
    let (repo_detected, branch, head_commit) = detect_git_repository();
    let mut capabilities = vec![
        "bundle_export".to_string(),
        "bundle_import".to_string(),
        "commit_message_draft".to_string(),
        "git_refs_listing".to_string(),
        "semantic_model_diff".to_string(),
    ];
    let mut notes = Vec::new();

    if repo_detected {
        capabilities.push("repository_detected".to_string());
        notes.push("Repository .git rilevato: disponibile workflow di collaborazione locale".to_string());
    } else {
        notes.push("Nessuna cartella .git rilevata nella working directory server".to_string());
    }

    if branch.is_none() {
        notes.push("Branch non determinabile automaticamente".to_string());
    }

    track_api_request(&state, "/api/collab/status", true);
    Json(CollabStatusResponse {
        repository_detected: repo_detected,
        branch,
        head_commit,
        capabilities,
        notes,
    })
}

async fn collab_git_refs(State(state): State<AppState>) -> Json<GitRefsResponse> {
    let (repository_detected, branches, tags, notes) = list_git_refs();
    track_api_request(&state, "/api/collab/git-refs", true);
    Json(GitRefsResponse {
        repository_detected,
        branches,
        tags,
        notes,
    })
}

async fn generate_commit_draft(
    State(state): State<AppState>,
    Json(mut req): Json<CommitDraftRequest>,
) -> Json<CommitDraftResponse> {
    apply_defaults(&mut req.model, &state.defaults);
    ensure_model_metadata(&mut req.model);

    let mut stats = HashMap::new();
    stats.insert("topics".to_string(), req.model.topics.len());
    stats.insert("classes".to_string(), req.model.classes.len());
    stats.insert("attributes".to_string(), req.model.classes.iter().map(|c| c.attributes.len()).sum());
    stats.insert("domains".to_string(), req.model.domains.len());
    stats.insert("associations".to_string(), req.model.associations.len());
    stats.insert("catalog_instances".to_string(), req.model.catalog_instances.len());
    stats.insert(
        "catalog_validation_presets".to_string(),
        req.model.catalog_validation_presets.len(),
    );
    stats.insert("prose_sections".to_string(), req.model.prose_sections.len());

    let focus = req.focus.unwrap_or_else(|| "model-update".to_string());
    let model_name = req.model.model_name.trim();
    let model_name = if model_name.is_empty() {
        "Model"
    } else {
        model_name
    };

    let title = format!(
        "feat(imc): {} [{}]",
        focus,
        sanitize_commit_subject_fragment(model_name)
    );

    let body = format!(
        "Model: {}\nVersion: {}\n\nSummary:\n- Topics: {}\n- Classes: {}\n- Attributes: {}\n- Domains: {}\n- Associations: {}\n- Catalog instances: {}\n- Catalog presets: {}\n- Prose sections: {}",
        model_name,
        req.model.model_version,
        stats.get("topics").copied().unwrap_or(0),
        stats.get("classes").copied().unwrap_or(0),
        stats.get("attributes").copied().unwrap_or(0),
        stats.get("domains").copied().unwrap_or(0),
        stats.get("associations").copied().unwrap_or(0),
        stats.get("catalog_instances").copied().unwrap_or(0),
        stats
            .get("catalog_validation_presets")
            .copied()
            .unwrap_or(0),
        stats.get("prose_sections").copied().unwrap_or(0)
    );

    track_api_request(&state, "/api/collab/commit-draft", true);
    Json(CommitDraftResponse { title, body, stats })
}

async fn compare_bundle_with_model(
    State(state): State<AppState>,
    Json(mut req): Json<ModelDiffRequest>,
) -> Json<ModelDiffResponse> {
    apply_defaults(&mut req.current_model, &state.defaults);
    ensure_model_metadata(&mut req.current_model);
    apply_defaults(&mut req.bundle_model, &state.defaults);
    ensure_model_metadata(&mut req.bundle_model);

    let (summary, details) = semantic_model_diff(&req.current_model, &req.bundle_model);
    let same = details.is_empty() && summary.values().all(|v| *v == 0);

    track_api_request(&state, "/api/collab/model-diff", true);
    Json(ModelDiffResponse {
        same,
        summary,
        details,
    })
}

async fn new_project(
    State(state): State<AppState>,
    Json(req): Json<NewProjectRequest>,
) -> Json<InterlisProject> {
    let project_name = req
        .project_name
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| "NewProject".to_string());

    let model_name = req
        .model_name
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| state.defaults.model_name.clone());

    let language = req
        .language
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| state.defaults.language.clone());

    track_api_request(&state, "/api/project/new", true);
    Json(InterlisProject::empty(project_name, model_name, language))
}

async fn import_imcp(
    State(state): State<AppState>,
    Json(mut project): Json<InterlisProject>,
) -> Json<ProjectImportResponse> {
    if project.project_name.trim().is_empty() {
        project.project_name = "ImportedProject".to_string();
    }

    apply_defaults(&mut project.model, &state.defaults);
    ensure_model_metadata(&mut project.model);
    let issues = validate(&project.model);
    let valid = !issues.iter().any(|i| i.level == "error");

    track_api_request(&state, "/api/project/import/imcp", true);
    Json(ProjectImportResponse {
        valid,
        project,
        issues,
    })
}

async fn export_imcp(
    State(state): State<AppState>,
    Json(mut project): Json<InterlisProject>,
) -> Result<impl IntoResponse, (StatusCode, Json<ValidationResponse>)> {
    if project.project_name.trim().is_empty() {
        project.project_name = "Project".to_string();
    }

    apply_defaults(&mut project.model, &state.defaults);
    ensure_model_metadata(&mut project.model);
    let issues = validate(&project.model);

    let file_name = format!("{}.imcp", sanitize_file_stem(&project.project_name));
    let project_json = match serde_json::to_string_pretty(&project) {
        Ok(v) => v,
        Err(_) => {
            track_api_request(&state, "/api/project/export/imcp", false);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ValidationResponse {
                    valid: false,
                    issues: vec![ValidationIssue {
                        level: "error",
                        code: "IMCP_SERIALIZATION_FAILED",
                        message: "Could not serialize project to IMCP JSON".to_string(),
                        node_path: None,
                        line: None,
                        column: None,
                    }],
                }),
            ));
        }
    };

    let response = ProjectExportResponse {
        file_name,
        project_json,
        issues,
    };

    track_api_request(&state, "/api/project/export/imcp", true);

    Ok((
        [(header::CONTENT_TYPE, "application/json; charset=utf-8")],
        Json(response),
    ))
}

async fn import_ili(
    State(state): State<AppState>,
    Json(req): Json<ImportIliRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ValidationResponse>)> {
    let trimmed = req.interlis.trim();
    let original_interlis = trimmed.to_string();
    if trimmed.is_empty() {
        track_api_request(&state, "/api/model/import/ili", false);
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(ValidationResponse {
                valid: false,
                issues: vec![error_issue("ILI_INPUT_EMPTY", "INTERLIS input is empty")],
            }),
        ));
    }

    let mut model = match from_interlis(&original_interlis) {
        Ok(m) => m,
        Err(parse_error) => {
            track_api_request(&state, "/api/model/import/ili", false);
            return Err((
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(ValidationResponse {
                    valid: false,
                    issues: vec![ValidationIssue {
                        level: "error",
                        code: "ILI_PARSE_FAILED",
                        message: parse_error.message,
                        node_path: Some("model".to_string()),
                        line: Some(parse_error.line),
                        column: Some(parse_error.column),
                    }],
                }),
            ));
        }
    };

    apply_defaults(&mut model, &state.defaults);
    ensure_model_metadata(&mut model);

    if let Some(current) = req.current_model {
        let same_name = current.model_name.trim() == model.model_name.trim();
        let same_version = current.model_version.trim() == model.model_version.trim();
        let same_md5 = !current.model_md5.trim().is_empty() && current.model_md5 == model.model_md5;
        let allow_overwrite = req
            .on_conflict
            .as_deref()
            .map(|v| v.eq_ignore_ascii_case("overwrite"))
            .unwrap_or(false);

        if same_name && !(same_version && same_md5) && !allow_overwrite {
            track_api_request(&state, "/api/model/import/ili", false);
            return Err((
                StatusCode::CONFLICT,
                Json(ValidationResponse {
                    valid: false,
                    issues: vec![error_issue(
                        "ILI_MODEL_CONFLICT",
                        "Model with same name already exists but version/md5 differ; pass on_conflict=overwrite to replace",
                    )],
                }),
            ));
        }
    }

    let issues = validate(&model);
    let generated_interlis = model.to_interlis();
    let mut external_issues = run_toolchain_model_validation(
        &state.tooling,
        &model,
        &original_interlis,
        "import-ili-original",
    );
    let mut regenerated_external_issues = run_toolchain_model_validation(
        &state.tooling,
        &model,
        &generated_interlis,
        "import-ili-regenerated",
    );
    external_issues.append(&mut regenerated_external_issues);

    let mut merged_issues = issues;
    merged_issues.append(&mut external_issues);

    let valid = !merged_issues.iter().any(|i| i.level == "error");
    let response = ImportIliResponse {
        valid,
        model,
        issues: merged_issues,
    };

    track_api_request(&state, "/api/model/import/ili", valid);

    Ok((
        [(header::CONTENT_TYPE, "application/json; charset=utf-8")],
        Json(response),
    ))
}

async fn import_xtf(State(state): State<AppState>, Json(_req): Json<TransferRequest>) -> Json<TransferResponse> {
    import_xtf_with_toolchain(state, _req)
}

async fn export_xtf(State(state): State<AppState>, Json(req): Json<TransferRequest>) -> Json<TransferResponse> {
    export_xtf_with_toolchain(state, req)
}

async fn validate_model(
    State(state): State<AppState>,
    Json(mut model): Json<InterlisModel>,
) -> Json<ValidationResponse> {
    apply_defaults(&mut model, &state.defaults);
    ensure_model_metadata(&mut model);
    let mut issues = validate(&model);
    let generated_interlis = model.to_interlis();
    let mut external_issues = run_toolchain_model_validation(
        &state.tooling,
        &model,
        &generated_interlis,
        "validate-model",
    );
    issues.append(&mut external_issues);
    let valid = !issues.iter().any(|i| i.level == "error");
    track_api_request(&state, "/api/model/validate", true);
    Json(ValidationResponse { valid, issues })
}

async fn export_ili(
    State(state): State<AppState>,
    Json(mut model): Json<InterlisModel>,
) -> Result<impl IntoResponse, (StatusCode, Json<ValidationResponse>)> {
    apply_defaults(&mut model, &state.defaults);
    ensure_model_metadata(&mut model);
    let issues = validate(&model);

    if issues.iter().any(|i| i.level == "error") {
        track_api_request(&state, "/api/model/export/ili", false);
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(ValidationResponse {
                valid: false,
                issues,
            }),
        ));
    }

    let interlis = model.to_interlis();
    let mut external_issues = run_toolchain_model_validation(
        &state.tooling,
        &model,
        &interlis,
        "export-ili",
    );

    if external_issues.iter().any(|i| i.level == "error") {
        let mut merged = issues;
        merged.append(&mut external_issues);
        track_api_request(&state, "/api/model/export/ili", false);
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(ValidationResponse {
                valid: false,
                issues: merged,
            }),
        ));
    }

    let mut merged_issues = issues;
    merged_issues.append(&mut external_issues);
    let file_name = format!("{}.ili", sanitize_file_stem(&model.model_name));

    let response = ExportResponse {
        file_name,
        interlis,
        issues: merged_issues,
    };

    track_api_request(&state, "/api/model/export/ili", true);

    Ok((
        [(header::CONTENT_TYPE, "application/json; charset=utf-8")],
        Json(response),
    ))
}

async fn export_docs_markdown(
    State(state): State<AppState>,
    Json(mut model): Json<InterlisModel>,
) -> Result<impl IntoResponse, (StatusCode, Json<ValidationResponse>)> {
    apply_defaults(&mut model, &state.defaults);
    ensure_model_metadata(&mut model);
    let issues = validate(&model);

    if issues.iter().any(|i| i.level == "error") {
        track_api_request(&state, "/api/docs/export/markdown", false);
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(ValidationResponse {
                valid: false,
                issues,
            }),
        ));
    }

    let markdown = model_to_markdown(&model);
    let file_name = format!("{}-documentation.md", sanitize_file_stem(&model.model_name));
    let response = DocsExportResponse {
        file_name,
        markdown,
        issues,
    };

    track_api_request(&state, "/api/docs/export/markdown", true);

    Ok((
        [(header::CONTENT_TYPE, "application/json; charset=utf-8")],
        Json(response),
    ))
}

fn now_unix_s() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn track_api_request(state: &AppState, endpoint: &str, success: bool) {
    if let Ok(mut metrics) = state.metrics.lock() {
        metrics.total_requests += 1;
        if !success {
            metrics.failed_requests += 1;
        }
        let key = endpoint.to_string();
        let current = metrics.by_endpoint.get(&key).copied().unwrap_or(0);
        metrics.by_endpoint.insert(key, current + 1);
    }
}

fn detect_git_repository() -> (bool, Option<String>, Option<String>) {
    let cwd = match env::current_dir() {
        Ok(v) => v,
        Err(_) => return (false, None, None),
    };

    let git_dir = cwd.join(".git");
    if !git_dir.is_dir() {
        return (false, None, None);
    }

    let head_path = git_dir.join("HEAD");
    let head_raw = match fs::read_to_string(&head_path) {
        Ok(v) => v,
        Err(_) => return (true, None, None),
    };

    let head_trimmed = head_raw.trim();
    if let Some(reference) = head_trimmed.strip_prefix("ref:") {
        let ref_rel = reference.trim();
        let branch = ref_rel.rsplit('/').next().map(|s| s.to_string());
        let ref_path = git_dir.join(Path::new(ref_rel));
        let sha = fs::read_to_string(ref_path).ok().map(|v| v.trim().to_string());
        return (true, branch, sha);
    }

    (true, Some("detached".to_string()), Some(head_trimmed.to_string()))
}

fn list_git_refs() -> (bool, Vec<GitRefEntry>, Vec<GitRefEntry>, Vec<String>) {
    let cwd = match env::current_dir() {
        Ok(v) => v,
        Err(_) => {
            return (
                false,
                Vec::new(),
                Vec::new(),
                vec!["Working directory non disponibile".to_string()],
            );
        }
    };

    let git_dir = cwd.join(".git");
    if !git_dir.is_dir() {
        return (
            false,
            Vec::new(),
            Vec::new(),
            vec!["Nessuna cartella .git rilevata".to_string()],
        );
    }

    let mut branch_map: HashMap<String, String> = HashMap::new();
    let mut tag_map: HashMap<String, String> = HashMap::new();

    collect_refs_from_loose_dir(&git_dir.join("refs").join("heads"), "", &mut branch_map);
    collect_refs_from_loose_dir(&git_dir.join("refs").join("tags"), "", &mut tag_map);
    collect_refs_from_packed_file(&git_dir.join("packed-refs"), &mut branch_map, &mut tag_map);

    let mut branches: Vec<GitRefEntry> = branch_map
        .into_iter()
        .map(|(name, sha)| GitRefEntry { name, sha })
        .collect();
    let mut tags: Vec<GitRefEntry> = tag_map
        .into_iter()
        .map(|(name, sha)| GitRefEntry { name, sha })
        .collect();

    branches.sort_by(|a, b| a.name.cmp(&b.name));
    tags.sort_by(|a, b| a.name.cmp(&b.name));

    let notes = vec![format!(
        "Rilevati {} branch e {} tag",
        branches.len(),
        tags.len()
    )];
    (true, branches, tags, notes)
}

fn collect_refs_from_loose_dir(root: &Path, prefix: &str, out: &mut HashMap<String, String>) {
    let entries = match fs::read_dir(root) {
        Ok(v) => v,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        let full = if prefix.is_empty() {
            name.clone()
        } else {
            format!("{}/{}", prefix, name)
        };

        if path.is_dir() {
            collect_refs_from_loose_dir(&path, &full, out);
            continue;
        }

        if !path.is_file() {
            continue;
        }

        if let Ok(raw) = fs::read_to_string(&path) {
            let sha = raw.trim().to_string();
            if !sha.is_empty() {
                out.insert(full, sha);
            }
        }
    }
}

fn collect_refs_from_packed_file(
    packed_refs_path: &Path,
    branches: &mut HashMap<String, String>,
    tags: &mut HashMap<String, String>,
) {
    let raw = match fs::read_to_string(packed_refs_path) {
        Ok(v) => v,
        Err(_) => return,
    };

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('^') {
            continue;
        }
        let mut parts = trimmed.split_whitespace();
        let sha = match parts.next() {
            Some(v) => v,
            None => continue,
        };
        let full_ref = match parts.next() {
            Some(v) => v,
            None => continue,
        };

        if let Some(name) = full_ref.strip_prefix("refs/heads/") {
            branches.entry(name.to_string()).or_insert_with(|| sha.to_string());
        } else if let Some(name) = full_ref.strip_prefix("refs/tags/") {
            tags.entry(name.to_string()).or_insert_with(|| sha.to_string());
        }
    }
}

fn semantic_model_diff(
    current_model: &InterlisModel,
    bundle_model: &InterlisModel,
) -> (HashMap<String, i64>, Vec<String>) {
    let mut summary = HashMap::new();
    let mut details = Vec::new();

    diff_named_collection(
        "topics",
        current_model.topics.iter().map(|v| v.name.as_str()),
        bundle_model.topics.iter().map(|v| v.name.as_str()),
        &mut summary,
        &mut details,
    );
    diff_named_collection(
        "classes",
        current_model.classes.iter().map(|v| v.name.as_str()),
        bundle_model.classes.iter().map(|v| v.name.as_str()),
        &mut summary,
        &mut details,
    );
    diff_named_collection(
        "domains",
        current_model.domains.iter().map(|v| v.name.as_str()),
        bundle_model.domains.iter().map(|v| v.name.as_str()),
        &mut summary,
        &mut details,
    );
    diff_named_collection(
        "associations",
        current_model.associations.iter().map(|v| v.name.as_str()),
        bundle_model.associations.iter().map(|v| v.name.as_str()),
        &mut summary,
        &mut details,
    );
    diff_named_collection(
        "prose_sections",
        current_model.prose_sections.iter().map(|v| v.title.as_str()),
        bundle_model.prose_sections.iter().map(|v| v.title.as_str()),
        &mut summary,
        &mut details,
    );
    diff_named_collection(
        "catalog_presets",
        current_model
            .catalog_validation_presets
            .iter()
            .map(|v| v.name.as_str()),
        bundle_model
            .catalog_validation_presets
            .iter()
            .map(|v| v.name.as_str()),
        &mut summary,
        &mut details,
    );

    let current_catalog_keys: Vec<String> = current_model
        .catalog_instances
        .iter()
        .map(|row| format!("{}::{}", row.catalog.trim(), row.code.trim()))
        .collect();
    let bundle_catalog_keys: Vec<String> = bundle_model
        .catalog_instances
        .iter()
        .map(|row| format!("{}::{}", row.catalog.trim(), row.code.trim()))
        .collect();
    diff_named_collection(
        "catalog_instances",
        current_catalog_keys.iter().map(|v| v.as_str()),
        bundle_catalog_keys.iter().map(|v| v.as_str()),
        &mut summary,
        &mut details,
    );

    let current_attr_map: HashMap<String, HashSet<String>> = current_model
        .classes
        .iter()
        .map(|class_def| {
            (
                class_def.name.to_ascii_lowercase(),
                class_def
                    .attributes
                    .iter()
                    .map(|attr| attr.name.to_ascii_lowercase())
                    .collect(),
            )
        })
        .collect();
    let bundle_attr_map: HashMap<String, HashSet<String>> = bundle_model
        .classes
        .iter()
        .map(|class_def| {
            (
                class_def.name.to_ascii_lowercase(),
                class_def
                    .attributes
                    .iter()
                    .map(|attr| attr.name.to_ascii_lowercase())
                    .collect(),
            )
        })
        .collect();

    let mut attr_added = 0_i64;
    let mut attr_removed = 0_i64;
    for (class_name, current_attrs) in &current_attr_map {
        if let Some(bundle_attrs) = bundle_attr_map.get(class_name) {
            for attr in bundle_attrs {
                if !current_attrs.contains(attr) {
                    attr_added += 1;
                    details.push(format!(
                        "class '{}' attribute aggiunto nel bundle: '{}'",
                        class_name, attr
                    ));
                }
            }
            for attr in current_attrs {
                if !bundle_attrs.contains(attr) {
                    attr_removed += 1;
                    details.push(format!(
                        "class '{}' attribute rimosso nel bundle: '{}'",
                        class_name, attr
                    ));
                }
            }
        }
    }
    summary.insert("attributes_added".to_string(), attr_added);
    summary.insert("attributes_removed".to_string(), attr_removed);

    if current_model.model_version.trim() != bundle_model.model_version.trim() {
        details.push(format!(
            "model_version diversa: current='{}' vs bundle='{}'",
            current_model.model_version.trim(),
            bundle_model.model_version.trim()
        ));
    }
    if current_model.model_md5.trim() != bundle_model.model_md5.trim() {
        details.push("model_md5 differente tra current e bundle".to_string());
    }

    (summary, details)
}

fn diff_named_collection<'a>(
    label: &str,
    current_iter: impl Iterator<Item = &'a str>,
    bundle_iter: impl Iterator<Item = &'a str>,
    summary: &mut HashMap<String, i64>,
    details: &mut Vec<String>,
) {
    let current_set: HashSet<String> = current_iter
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(|v| v.to_ascii_lowercase())
        .collect();
    let bundle_set: HashSet<String> = bundle_iter
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(|v| v.to_ascii_lowercase())
        .collect();

    let mut added: Vec<String> = bundle_set.difference(&current_set).cloned().collect();
    let mut removed: Vec<String> = current_set.difference(&bundle_set).cloned().collect();
    added.sort();
    removed.sort();

    summary.insert(format!("{}_added", label), added.len() as i64);
    summary.insert(format!("{}_removed", label), removed.len() as i64);

    for item in added {
        details.push(format!("{} aggiunto nel bundle: '{}'", label, item));
    }
    for item in removed {
        details.push(format!("{} rimosso nel bundle: '{}'", label, item));
    }
}

fn sanitize_commit_subject_fragment(value: &str) -> String {
    let cleaned: String = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '-'
            }
        })
        .collect();
    let compact: String = cleaned.trim_matches('-').chars().take(42).collect();
    if compact.is_empty() {
        "model".to_string()
    } else {
        compact
    }
}

fn sanitize_file_stem(stem: &str) -> String {
    let cleaned: String = stem
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect();

    if cleaned.trim_matches('_').is_empty() {
        "model".to_string()
    } else {
        cleaned
    }
}

fn apply_defaults(model: &mut InterlisModel, defaults: &Defaults) {
    if model.model_name.trim().is_empty() {
        model.model_name = defaults.model_name.clone();
    }
    if model.language.trim().is_empty() {
        model.language = defaults.language.clone();
    }
    if model.model_version.trim().is_empty() {
        model.model_version = "INTERLIS 2.4".to_string();
    }
    if model.model_at.trim().is_empty() {
        model.model_at = "https://intelligeo.ch/imc".to_string();
    }
}

fn is_supported_interlis_version(value: &str) -> bool {
    let normalized = value.trim().trim_end_matches(';').trim();
    normalized.eq_ignore_ascii_case("INTERLIS 2.3")
        || normalized.eq_ignore_ascii_case("INTERLIS 2.4")
}

fn ensure_model_metadata(model: &mut InterlisModel) {
    if model.model_md5.trim().is_empty() {
        model.model_md5 = format!("{:x}", md5::compute(model.to_interlis().as_bytes()));
    }
}

fn run_toolchain_model_validation(
    tooling: &ToolchainConfig,
    model: &InterlisModel,
    interlis: &str,
    node_path: &str,
) -> Vec<ValidationIssue> {
    let req = ToolRequest {
        request_id: format!("{}-{}", node_path, now_unix_s()),
        model_name: model.model_name.clone(),
        interlis_text: interlis.to_string(),
        transfer_text: None,
        transfer_format: None,
        instances_json: None,
        timeout_ms: None,
        strict: false,
    };

    match toolchain_adapter::execute(tooling, ToolCommand::ValidateModel, &req) {
        Ok(resp) => map_tool_issues(resp.issues),
        Err(err) => vec![tool_exec_error_issue(err, node_path, tooling.ili2c.strict)],
    }
}

fn import_xtf_with_toolchain(state: AppState, req: TransferRequest) -> Json<TransferResponse> {
    if req.content.trim().is_empty() {
        track_api_request(&state, "/api/instance/import/xtf", false);
        return Json(TransferResponse {
            valid: false,
            issues: vec![error_issue("XTF_INPUT_EMPTY", "XTF/XML input is empty")],
        });
    }

    let mut model = req.model.unwrap_or_else(|| InterlisModel {
        model_name: state.defaults.model_name.clone(),
        language: state.defaults.language.clone(),
        model_at: "https://intelligeo.ch/imc".to_string(),
        model_model_version: String::new(),
        model_version: "INTERLIS 2.4".to_string(),
        model_md5: String::new(),
        domains: Vec::new(),
        associations: Vec::new(),
        topics: Vec::new(),
        classes: Vec::new(),
        prose_sections: Vec::new(),
        catalog_instances: Vec::new(),
        catalog_validation_presets: Vec::new(),
    });
    apply_defaults(&mut model, &state.defaults);
    ensure_model_metadata(&mut model);

    let interlis = model.to_interlis();
    let tool_req = ToolRequest {
        request_id: format!("import-xtf-{}", now_unix_s()),
        model_name: model.model_name.clone(),
        interlis_text: interlis,
        transfer_text: Some(req.content),
        transfer_format: Some("xtf".to_string()),
        instances_json: None,
        timeout_ms: None,
        strict: false,
    };

    let mut issues = match toolchain_adapter::execute(&state.tooling, ToolCommand::ValidateTransfer, &tool_req) {
        Ok(resp) => map_tool_issues(resp.issues),
        Err(err) => vec![tool_exec_error_issue(err, "xtf", state.tooling.ilivalidator.strict)],
    };

    let valid = !issues.iter().any(|i| i.level == "error");
    track_api_request(&state, "/api/instance/import/xtf", valid);
    Json(TransferResponse {
        valid,
        issues: std::mem::take(&mut issues),
    })
}

fn export_xtf_with_toolchain(state: AppState, req: TransferRequest) -> Json<TransferResponse> {
    let mut model = req.model.unwrap_or_else(|| InterlisModel {
        model_name: state.defaults.model_name.clone(),
        language: state.defaults.language.clone(),
        model_at: "https://intelligeo.ch/imc".to_string(),
        model_model_version: String::new(),
        model_version: "INTERLIS 2.4".to_string(),
        model_md5: String::new(),
        domains: Vec::new(),
        associations: Vec::new(),
        topics: Vec::new(),
        classes: Vec::new(),
        prose_sections: Vec::new(),
        catalog_instances: Vec::new(),
        catalog_validation_presets: Vec::new(),
    });
    apply_defaults(&mut model, &state.defaults);
    ensure_model_metadata(&mut model);

    let tool_req = ToolRequest {
        request_id: format!("export-xtf-{}", now_unix_s()),
        model_name: model.model_name.clone(),
        interlis_text: model.to_interlis(),
        transfer_text: None,
        transfer_format: Some("xtf".to_string()),
        instances_json: Some(req.content),
        timeout_ms: None,
        strict: false,
    };

    let mut issues = match toolchain_adapter::execute(&state.tooling, ToolCommand::ExportTransfer, &tool_req) {
        Ok(resp) => map_tool_issues(resp.issues),
        Err(err) => vec![tool_exec_error_issue(err, "xtf", state.tooling.iox_ili.strict)],
    };

    let valid = !issues.iter().any(|i| i.level == "error");
    track_api_request(&state, "/api/instance/export/xtf", valid);
    Json(TransferResponse {
        valid,
        issues: std::mem::take(&mut issues),
    })
}

fn map_tool_issues(tool_issues: Vec<ToolIssue>) -> Vec<ValidationIssue> {
    tool_issues
        .into_iter()
        .map(|item| ValidationIssue {
            level: match item.severity {
                "error" | "fatal" => "error",
                "warning" | "warn" => "warning",
                _ => "info",
            },
            code: item.code,
            message: item.message,
            node_path: item.path,
            line: item.line,
            column: item.column,
        })
        .collect()
}

fn tool_exec_error_issue(err: ToolExecError, node_path: &str, strict: bool) -> ValidationIssue {
    match err {
        ToolExecError::Timeout => ValidationIssue {
            level: if strict { "error" } else { "warning" },
            code: "TOOL_TIMEOUT",
            message: "tool execution timed out".to_string(),
            node_path: Some(node_path.to_string()),
            line: None,
            column: None,
        },
        ToolExecError::Io(message) => ValidationIssue {
            level: if strict { "error" } else { "warning" },
            code: "TOOL_IO_ERROR",
            message,
            node_path: Some(node_path.to_string()),
            line: None,
            column: None,
        },
        ToolExecError::Process(message) => ValidationIssue {
            level: if strict { "error" } else { "warning" },
            code: "TOOL_UNAVAILABLE",
            message,
            node_path: Some(node_path.to_string()),
            line: None,
            column: None,
        },
    }
}

fn setup_file_logger() -> Result<(NonBlocking, WorkerGuard, String), String> {
    let log_dir = resolve_user_log_dir()?;
    fs::create_dir_all(&log_dir).map_err(|e| format!("unable to create log dir {}: {}", log_dir.display(), e))?;

    let file_appender = tracing_appender::rolling::daily(&log_dir, "interlis-composer-rs.log");
    let (writer, guard) = tracing_appender::non_blocking(file_appender);

    let full_path = log_dir
        .join("interlis-composer-rs.log")
        .to_string_lossy()
        .to_string();
    Ok((writer, guard, full_path))
}

fn resolve_user_log_dir() -> Result<PathBuf, String> {
    if cfg!(target_os = "windows") {
        if let Some(home) = env::var_os("USERPROFILE") {
            return Ok(PathBuf::from(home).join(".versatili"));
        }
        let drive = env::var_os("HOMEDRIVE");
        let path = env::var_os("HOMEPATH");
        if let (Some(drive), Some(path)) = (drive, path) {
            return Ok(PathBuf::from(format!(
                "{}{}",
                drive.to_string_lossy(),
                path.to_string_lossy()
            ))
            .join(".versatili"));
        }
    }

    if let Some(home) = env::var_os("HOME") {
        return Ok(PathBuf::from(home).join(".versatili"));
    }

    env::current_dir()
        .map(|dir| dir.join(".versatili"))
        .map_err(|e| format!("unable to resolve user directory: {}", e))
}

fn validate(model: &InterlisModel) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    if !is_supported_interlis_version(&model.model_version) {
        issues.push(error_issue_at(
            "MODEL_VERSION_UNSUPPORTED",
            "Unsupported model version: supported versions are INTERLIS 2.3 and INTERLIS 2.4",
            Some("model".to_string()),
        ));
    }

    if !is_identifier(&model.model_name) {
        issues.push(error_issue_at(
            "MODEL_NAME_INVALID",
            "Model name is invalid: INTERLIS identifiers must start with a letter and then contain only A-Z, a-z, 0-9 or '_' (no spaces)",
            Some("model".to_string()),
        ));
    }

    if model.classes.is_empty() {
        issues.push(warn_issue_at(
            "NO_CLASSES",
            "Model has no classes; exported INTERLIS will be mostly empty",
            Some("model".to_string()),
        ));
    }

    validate_topics(&model.topics, &mut issues);
    validate_domains(&model.domains, &mut issues);
    validate_associations(&model.associations, &model.classes, &mut issues);
    validate_inheritance(&model.classes, &mut issues);
    validate_prose_sections(&model.prose_sections, &mut issues);
    validate_catalog_validation_presets(
        &model.catalog_validation_presets,
        &model.classes,
        &model.domains,
        &mut issues,
    );
    validate_catalog_instances(
        &model.catalog_instances,
        &model.classes,
        &model.domains,
        &model.catalog_validation_presets,
        &mut issues,
    );
    validate_duplicates(&model.classes, &model.domains, &model.associations, &mut issues);

    for class in &model.classes {
        validate_class(class, &mut issues);
    }

    issues
}

fn validate_prose_sections(sections: &[ProseSection], issues: &mut Vec<ValidationIssue>) {
    let mut seen = HashSet::new();
    for section in sections {
        let title = section.title.trim();
        let section_path = format!("prose:{}", title);
        if title.is_empty() {
            issues.push(error_issue_at(
                "PROSE_TITLE_EMPTY",
                "Prose section title cannot be empty",
                Some("prose".to_string()),
            ));
            continue;
        }

        if !seen.insert(title.to_ascii_lowercase()) {
            issues.push(error_issue_at(
                "PROSE_TITLE_DUPLICATE",
                &format!("Prose section '{}' appears multiple times", title),
                Some(section_path.clone()),
            ));
        }

        if section.content.trim().is_empty() {
            issues.push(warn_issue_at(
                "PROSE_CONTENT_EMPTY",
                &format!("Prose section '{}' has empty content", title),
                Some(section_path),
            ));
        }
    }
}

fn validate_catalog_validation_presets(
    presets: &[CatalogValidationPreset],
    classes: &[Class],
    domains: &[Domain],
    issues: &mut Vec<ValidationIssue>,
) {
    let class_names: HashSet<String> = classes.iter().map(|c| c.name.to_ascii_lowercase()).collect();
    let class_by_name: HashMap<String, &Class> = classes
        .iter()
        .map(|c| (c.name.to_ascii_lowercase(), c))
        .collect();
    let domain_by_name: HashMap<String, &Domain> = domains
        .iter()
        .map(|d| (d.name.to_ascii_lowercase(), d))
        .collect();
    let mut seen_preset_names = HashSet::new();

    for preset in presets {
        let name = preset.name.trim();
        let class_name = preset.target_class.trim();
        let preset_path = format!("catalog-preset:{}", name);
        if name.is_empty() {
            issues.push(error_issue_at(
                "CATALOG_PRESET_NAME_EMPTY",
                "Catalog validation preset name cannot be empty",
                Some("catalog-preset".to_string()),
            ));
            continue;
        }

        if !seen_preset_names.insert(name.to_ascii_lowercase()) {
            issues.push(error_issue_at(
                "CATALOG_PRESET_NAME_DUPLICATE",
                &format!("Catalog validation preset '{}' appears multiple times", name),
                Some(preset_path.clone()),
            ));
        }

        if preset.version == 0 {
            issues.push(error_issue_at(
                "CATALOG_PRESET_VERSION_INVALID",
                &format!(
                    "Catalog validation preset '{}' has invalid version '{}'; minimum is 1",
                    name, preset.version
                ),
                Some(preset_path.clone()),
            ));
        }

        if !preset.owner.trim().is_empty() && !is_valid_preset_actor(preset.owner.trim()) {
            issues.push(error_issue_at(
                "CATALOG_PRESET_OWNER_INVALID",
                &format!(
                    "Catalog validation preset '{}' has invalid owner '{}'; allowed pattern is [A-Za-z0-9_.@-] with length 3..64",
                    name,
                    preset.owner.trim()
                ),
                Some(preset_path.clone()),
            ));
        }

        let review_status = preset.review_status.trim().to_ascii_lowercase();
        if !["draft", "approved", "deprecated"].contains(&review_status.as_str()) {
            issues.push(error_issue_at(
                "CATALOG_PRESET_REVIEW_STATUS_INVALID",
                &format!(
                    "Catalog validation preset '{}' has invalid review_status '{}'",
                    name,
                    preset.review_status.trim()
                ),
                Some(preset_path.clone()),
            ));
        }

        if review_status == "approved" {
            if preset.approved_by.trim().is_empty() || preset.approved_at.trim().is_empty() {
                issues.push(error_issue_at(
                    "CATALOG_PRESET_APPROVAL_MISSING",
                    &format!(
                        "Catalog validation preset '{}' is approved but approved_by/approved_at are missing",
                        name
                    ),
                    Some(preset_path.clone()),
                ));
            }
            if !preset.approved_by.trim().is_empty() && !is_valid_preset_actor(preset.approved_by.trim()) {
                issues.push(error_issue_at(
                    "CATALOG_PRESET_APPROVER_INVALID",
                    &format!(
                        "Catalog validation preset '{}' has invalid approved_by '{}'; allowed pattern is [A-Za-z0-9_.@-] with length 3..64",
                        name,
                        preset.approved_by.trim()
                    ),
                    Some(preset_path.clone()),
                ));
            }
            if !preset.approved_at.trim().is_empty() && !is_iso_like_datetime(preset.approved_at.trim()) {
                issues.push(error_issue_at(
                    "CATALOG_PRESET_APPROVED_AT_INVALID",
                    &format!(
                        "Catalog validation preset '{}' has invalid approved_at '{}'; expected ISO datetime",
                        name,
                        preset.approved_at.trim()
                    ),
                    Some(preset_path.clone()),
                ));
            }
        } else if !preset.approved_by.trim().is_empty() || !preset.approved_at.trim().is_empty() {
            issues.push(warn_issue_at(
                "CATALOG_PRESET_APPROVAL_IGNORED",
                &format!(
                    "Catalog validation preset '{}' has approval metadata but review_status is '{}'; metadata will be ignored until approved",
                    name,
                    preset.review_status.trim()
                ),
                Some(preset_path.clone()),
            ));
        }

        if class_name.is_empty() {
            issues.push(error_issue_at(
                "CATALOG_PRESET_TARGET_CLASS_EMPTY",
                &format!("Catalog validation preset '{}' has empty target_class", name),
                Some(preset_path.clone()),
            ));
            continue;
        }

        if !class_names.contains(&class_name.to_ascii_lowercase()) {
            issues.push(error_issue_at(
                "CATALOG_PRESET_TARGET_CLASS_UNKNOWN",
                &format!(
                    "Catalog validation preset '{}' refers to unknown class '{}'",
                    name, class_name
                ),
                Some(preset_path.clone()),
            ));
        }
        let target_class_def = class_by_name.get(&class_name.to_ascii_lowercase()).copied();

        let mut seen_rule_names = HashSet::new();
        for rule in &preset.rules {
            let attr_name = rule.attribute.trim();
            let rule_path = format!("{}:{}", preset_path, attr_name);
            if attr_name.is_empty() {
                issues.push(error_issue_at(
                    "CATALOG_PRESET_RULE_ATTR_EMPTY",
                    &format!("Catalog validation preset '{}' has a rule with empty attribute", name),
                    Some(preset_path.clone()),
                ));
                continue;
            }

            if !seen_rule_names.insert(attr_name.to_ascii_lowercase()) {
                issues.push(error_issue_at(
                    "CATALOG_PRESET_RULE_ATTR_DUPLICATE",
                    &format!(
                        "Catalog validation preset '{}' defines '{}' multiple times",
                        name, attr_name
                    ),
                    Some(rule_path.clone()),
                ));
            }

            if let Some(class_def) = target_class_def {
                if !class_def
                    .attributes
                    .iter()
                    .any(|a| a.name.eq_ignore_ascii_case(attr_name))
                {
                    issues.push(warn_issue_at(
                        "CATALOG_PRESET_RULE_ATTR_NOT_IN_CLASS",
                        &format!(
                            "Catalog validation preset '{}' defines attribute '{}' which is not part of class '{}'",
                            name, attr_name, class_name
                        ),
                        Some(rule_path.clone()),
                    ));
                }
            }

            if !rule.expected_type.trim().is_empty() && !model::is_supported_type(&rule.expected_type) {
                issues.push(error_issue_at(
                    "CATALOG_PRESET_RULE_TYPE_UNSUPPORTED",
                    &format!(
                        "Catalog validation preset '{}' has unsupported expected_type '{}' for attribute '{}'",
                        name, rule.expected_type, attr_name
                    ),
                    Some(rule_path.clone()),
                ));
            }

            if !rule.domain_ref.trim().is_empty() {
                let domain_name = rule.domain_ref.trim().to_ascii_lowercase();
                if let Some(domain) = domain_by_name.get(&domain_name) {
                    if !rule.expected_type.trim().is_empty()
                        && !domain.r#type.eq_ignore_ascii_case(&rule.expected_type)
                    {
                        issues.push(warn_issue_at(
                            "CATALOG_PRESET_RULE_DOMAIN_TYPE_MISMATCH",
                            &format!(
                                "Catalog validation preset '{}' rule '{}' has expected_type '{}' but domain '{}' uses '{}'",
                                name,
                                attr_name,
                                rule.expected_type,
                                domain.name,
                                domain.r#type
                            ),
                            Some(rule_path),
                        ));
                    }
                } else {
                    issues.push(error_issue_at(
                        "CATALOG_PRESET_RULE_DOMAIN_UNKNOWN",
                        &format!(
                            "Catalog validation preset '{}' rule '{}' refers to unknown domain '{}'",
                            name,
                            attr_name,
                            rule.domain_ref
                        ),
                        Some(rule_path),
                    ));
                }
            }
        }
    }
}

fn validate_catalog_instances(
    instances: &[CatalogInstance],
    classes: &[Class],
    domains: &[Domain],
    presets: &[CatalogValidationPreset],
    issues: &mut Vec<ValidationIssue>,
) {
    let class_names: HashSet<&str> = classes.iter().map(|c| c.name.as_str()).collect();
    let class_by_name: HashMap<&str, &Class> = classes.iter().map(|c| (c.name.as_str(), c)).collect();
    let domain_by_name: HashMap<String, &Domain> = domains
        .iter()
        .map(|d| (d.name.to_ascii_lowercase(), d))
        .collect();
    let preset_by_name: HashMap<String, &CatalogValidationPreset> = presets
        .iter()
        .map(|p| (p.name.trim().to_ascii_lowercase(), p))
        .collect();
    let mut seen_keys = HashSet::new();

    for row in instances {
        let catalog = row.catalog.trim();
        let code = row.code.trim();
        let label = row.label.trim();
        let class_ref = row.class_ref.trim();
        let preset_name = row.validation_preset.trim();
        let path = format!("catalog:{}:{}", catalog, code);

        if catalog.is_empty() {
            issues.push(error_issue_at(
                "CATALOG_INSTANCE_CATALOG_EMPTY",
                "Catalog instance catalog cannot be empty",
                Some(path.clone()),
            ));
        }

        if code.is_empty() {
            issues.push(error_issue_at(
                "CATALOG_INSTANCE_CODE_EMPTY",
                "Catalog instance code cannot be empty",
                Some(path.clone()),
            ));
        } else if !is_identifier(code) {
            issues.push(error_issue_at(
                "CATALOG_INSTANCE_CODE_INVALID",
                &format!("Catalog instance code '{}' is invalid: INTERLIS identifiers must start with a letter and then contain only A-Z, a-z, 0-9 or '_' (no spaces)", code),
                Some(path.clone()),
            ));
        }

        if label.is_empty() {
            issues.push(warn_issue_at(
                "CATALOG_INSTANCE_LABEL_EMPTY",
                &format!("Catalog instance '{}:{}' has empty label", catalog, code),
                Some(path.clone()),
            ));
        }

        if class_ref.is_empty() {
            issues.push(warn_issue_at(
                "CATALOG_INSTANCE_CLASS_REF_EMPTY",
                &format!("Catalog instance '{}:{}' has no class_ref", catalog, code),
                Some(path.clone()),
            ));
        } else if !class_names.contains(class_ref) {
            issues.push(error_issue_at(
                "CATALOG_INSTANCE_CLASS_REF_UNKNOWN",
                &format!("Catalog instance '{}:{}' refers to unknown class_ref '{}'", catalog, code, class_ref),
                Some(path.clone()),
            ));
        } else if let Some(class_def) = class_by_name.get(class_ref) {
            let preset_for_row = if preset_name.is_empty() {
                let mut by_class = presets
                    .iter()
                    .filter(|p| p.target_class.trim().eq_ignore_ascii_case(class_ref));
                let first = by_class.next();
                if first.is_some() && by_class.next().is_some() {
                    issues.push(warn_issue_at(
                        "CATALOG_INSTANCE_PRESET_AMBIGUOUS",
                        &format!(
                            "Catalog instance '{}:{}' has multiple presets for class '{}'; set validation_preset explicitly",
                            catalog, code, class_ref
                        ),
                        Some(path.clone()),
                    ));
                    None
                } else {
                    first
                }
            } else if let Some(preset) = preset_by_name.get(&preset_name.to_ascii_lowercase()) {
                if !preset.target_class.trim().eq_ignore_ascii_case(class_ref) {
                    issues.push(error_issue_at(
                        "CATALOG_INSTANCE_PRESET_CLASS_MISMATCH",
                        &format!(
                            "Catalog instance '{}:{}' uses preset '{}' for class '{}' but row class_ref is '{}'",
                            catalog,
                            code,
                            preset.name,
                            preset.target_class,
                            class_ref
                        ),
                        Some(path.clone()),
                    ));
                }
                Some(*preset)
            } else {
                issues.push(error_issue_at(
                    "CATALOG_INSTANCE_PRESET_UNKNOWN",
                    &format!(
                        "Catalog instance '{}:{}' refers to unknown validation_preset '{}'",
                        catalog, code, preset_name
                    ),
                    Some(path.clone()),
                ));
                None
            };

            let rule_by_attr: HashMap<String, &model::CatalogValidationRule> = preset_for_row
                .map(|preset| {
                    preset
                        .rules
                        .iter()
                        .map(|r| (r.attribute.trim().to_ascii_lowercase(), r))
                        .collect()
                })
                .unwrap_or_default();

            let code_attr = class_def
                .attributes
                .iter()
                .find(|a| a.name.eq_ignore_ascii_case("code"));
            let label_attr = class_def
                .attributes
                .iter()
                .find(|a| a.name.eq_ignore_ascii_case("label"));

            if code_attr.is_none() {
                issues.push(error_issue_at(
                    "CATALOG_INSTANCE_CLASS_MISSING_CODE_ATTR",
                    &format!(
                        "Class '{}' referenced by '{}:{}' has no attribute named 'code'",
                        class_ref, catalog, code
                    ),
                    Some(path.clone()),
                ));
            }
            if label_attr.is_none() {
                issues.push(error_issue_at(
                    "CATALOG_INSTANCE_CLASS_MISSING_LABEL_ATTR",
                    &format!(
                        "Class '{}' referenced by '{}:{}' has no attribute named 'label'",
                        class_ref, catalog, code
                    ),
                    Some(path.clone()),
                ));
            }

            if let Some(code_attr) = code_attr {
                let code_type = code_attr.r#type.trim().to_ascii_uppercase();
                if code_type == "NUMERIC" && !is_numeric_literal(code) {
                    issues.push(error_issue_at(
                        "CATALOG_INSTANCE_CODE_TYPE_MISMATCH",
                        &format!(
                            "Catalog instance '{}:{}' has non-numeric code but class '{}.{}' is NUMERIC",
                            catalog, code, class_ref, code_attr.name
                        ),
                        Some(path.clone()),
                    ));
                }
            }

            if let Some(label_attr) = label_attr {
                let label_type = label_attr.r#type.trim().to_ascii_uppercase();
                if label_type != "TEXT" {
                    issues.push(warn_issue_at(
                        "CATALOG_INSTANCE_LABEL_TYPE_UNUSUAL",
                        &format!(
                            "Class '{}.{}' has type '{}'; TEXT is recommended for catalog labels",
                            class_ref, label_attr.name, label_attr.r#type
                        ),
                        Some(path.clone()),
                    ));
                }
            }

            for attr in &class_def.attributes {
                let value = catalog_instance_attr_value(row, attr.name.as_str());
                let rule = rule_by_attr.get(&attr.name.to_ascii_lowercase()).copied();
                let effective_mandatory = attr.mandatory || rule.map(|r| r.mandatory).unwrap_or(false);
                let effective_type = rule
                    .and_then(|r| {
                        let t = r.expected_type.trim();
                        if t.is_empty() {
                            None
                        } else {
                            Some(t.to_string())
                        }
                    })
                    .unwrap_or_else(|| attr.r#type.clone());

                if effective_mandatory {
                    if value.map(|v| v.trim().is_empty()).unwrap_or(true) {
                        issues.push(error_issue_at(
                            "CATALOG_INSTANCE_MANDATORY_ATTR_MISSING",
                            &format!(
                                "Catalog instance '{}:{}' is missing mandatory attribute '{}.{}'",
                                catalog, code, class_ref, attr.name
                            ),
                            Some(path.clone()),
                        ));
                        continue;
                    }
                }

                if let Some(v) = value {
                    if !value_matches_type(v, &effective_type) {
                        issues.push(error_issue_at(
                            "CATALOG_INSTANCE_ATTR_TYPE_MISMATCH",
                            &format!(
                                "Catalog instance '{}:{}' value for '{}.{}' is incompatible with type '{}'",
                                catalog, code, class_ref, attr.name, effective_type
                            ),
                            Some(path.clone()),
                        ));
                    }

                    if let Some(domain_name) = rule
                        .and_then(|r| {
                            let d = r.domain_ref.trim();
                            if d.is_empty() {
                                None
                            } else {
                                Some(d.to_string())
                            }
                        })
                    {
                        if let Some(domain) = domain_by_name.get(&domain_name.to_ascii_lowercase()) {
                            if !value_matches_type(v, &domain.r#type) {
                                issues.push(error_issue_at(
                                    "CATALOG_INSTANCE_ATTR_DOMAIN_TYPE_MISMATCH",
                                    &format!(
                                        "Catalog instance '{}:{}' value for '{}.{}' is incompatible with domain '{}' ({})",
                                        catalog, code, class_ref, attr.name, domain.name, domain.r#type
                                    ),
                                    Some(path.clone()),
                                ));
                            }
                        }
                    }
                }
            }

            for rule in rule_by_attr.values() {
                let attr_name = rule.attribute.trim();
                if attr_name.is_empty()
                    || class_def
                        .attributes
                        .iter()
                        .any(|a| a.name.eq_ignore_ascii_case(attr_name))
                {
                    continue;
                }

                let value = catalog_instance_attr_value(row, attr_name);
                if rule.mandatory && value.map(|v| v.trim().is_empty()).unwrap_or(true) {
                    issues.push(error_issue_at(
                        "CATALOG_INSTANCE_PRESET_MANDATORY_ATTR_MISSING",
                        &format!(
                            "Catalog instance '{}:{}' is missing mandatory preset attribute '{}'",
                            catalog, code, attr_name
                        ),
                        Some(path.clone()),
                    ));
                }

                if let Some(v) = value {
                    if !rule.expected_type.trim().is_empty() && !value_matches_type(v, &rule.expected_type) {
                        issues.push(error_issue_at(
                            "CATALOG_INSTANCE_PRESET_ATTR_TYPE_MISMATCH",
                            &format!(
                                "Catalog instance '{}:{}' value for preset attribute '{}' is incompatible with type '{}'",
                                catalog, code, attr_name, rule.expected_type
                            ),
                            Some(path.clone()),
                        ));
                    }

                    if !rule.domain_ref.trim().is_empty() {
                        if let Some(domain) = domain_by_name.get(&rule.domain_ref.trim().to_ascii_lowercase()) {
                            if !value_matches_type(v, &domain.r#type) {
                                issues.push(error_issue_at(
                                    "CATALOG_INSTANCE_PRESET_ATTR_DOMAIN_TYPE_MISMATCH",
                                    &format!(
                                        "Catalog instance '{}:{}' value for preset attribute '{}' is incompatible with domain '{}' ({})",
                                        catalog, code, attr_name, domain.name, domain.r#type
                                    ),
                                    Some(path.clone()),
                                ));
                            }
                        }
                    }
                }
            }
        }

        let key = format!("{}::{}", catalog.to_ascii_lowercase(), code.to_ascii_lowercase());
        if !catalog.is_empty() && !code.is_empty() && !seen_keys.insert(key) {
            issues.push(error_issue_at(
                "CATALOG_INSTANCE_DUPLICATE",
                &format!("Catalog instance '{}:{}' appears multiple times", catalog, code),
                Some(path),
            ));
        }
    }
}

fn model_to_markdown(model: &InterlisModel) -> String {
    let mut lines: Vec<String> = Vec::new();
    let topic_count = model.topics.len();
    let class_count = model.classes.len();
    let domain_count = model.domains.len();
    let assoc_count = model.associations.len();
    let prose_count = model.prose_sections.len();
    let catalog_count = model.catalog_instances.len();

    lines.push(format!("# {}", model.model_name));
    lines.push(String::new());
    lines.push(format!("- Language: {}", model.language));
    lines.push(format!("- Version: {}", model.model_version));
    lines.push(format!("- Topics: {}", topic_count));
    lines.push(format!("- Classes: {}", class_count));
    lines.push(format!("- Domains: {}", domain_count));
    lines.push(format!("- Associations: {}", assoc_count));
    lines.push(format!("- Prose sections: {}", prose_count));
    lines.push(format!("- Catalog instances: {}", catalog_count));
    lines.push(String::new());

    lines.push("## Index".to_string());
    lines.push("- [Topics](#topics)".to_string());
    lines.push("- [Classes](#classes)".to_string());
    lines.push("- [Domains](#domains)".to_string());
    lines.push("- [Associations](#associations)".to_string());
    lines.push("- [Prose Sections](#prose-sections)".to_string());
    lines.push("- [Catalog Instances](#catalog-instances)".to_string());
    lines.push(String::new());

    lines.push("## Topics".to_string());
    if model.topics.is_empty() {
        lines.push("- none".to_string());
    } else {
        for topic in &model.topics {
            lines.push(format!("- {}", topic.name));
            for subtopic in &topic.subtopics {
                lines.push(format!("  - {}", subtopic.name));
            }
        }
    }
    lines.push(String::new());

    lines.push("## Classes".to_string());
    if model.classes.is_empty() {
        lines.push("- none".to_string());
    } else {
        for class in &model.classes {
            let class_anchor = slugify_anchor(&class.name);
            lines.push(format!("<a id=\"class-{}\"></a>", class_anchor));
            lines.push(format!("### {}", class.name));
            if class.attributes.is_empty() {
                lines.push("- no attributes".to_string());
            } else {
                for attr in &class.attributes {
                    let mandatory = if attr.mandatory { " (mandatory)" } else { "" };
                    lines.push(format!("- {}: {}{}", attr.name, attr.r#type, mandatory));
                }
            }
            lines.push(String::new());
        }
    }

    lines.push("## Domains".to_string());
    if model.domains.is_empty() {
        lines.push("- none".to_string());
    } else {
        for domain in &model.domains {
            lines.push(format!("- {}: {}", domain.name, domain.r#type));
        }
    }
    lines.push(String::new());

    lines.push("## Associations".to_string());
    if model.associations.is_empty() {
        lines.push("- none".to_string());
    } else {
        for assoc in &model.associations {
            let from_link = format!("[{}](#class-{})", assoc.from_class, slugify_anchor(&assoc.from_class));
            let to_link = format!("[{}](#class-{})", assoc.to_class, slugify_anchor(&assoc.to_class));
            lines.push(format!(
                "- {}: {} -> {} ({})",
                assoc.name, from_link, to_link, assoc.cardinality
            ));
        }
    }
    lines.push(String::new());

    lines.push("## Prose Sections".to_string());
    if model.prose_sections.is_empty() {
        lines.push("- none".to_string());
    } else {
        for section in &model.prose_sections {
            let prose_anchor = slugify_anchor(&section.title);
            lines.push(format!("<a id=\"prose-{}\"></a>", prose_anchor));
            lines.push(format!("### {}", section.title));
            lines.push(section.content.clone());
            lines.push(String::new());
        }
    }

    lines.push("## Catalog Instances".to_string());
    if model.catalog_instances.is_empty() {
        lines.push("- none".to_string());
    } else {
        for row in &model.catalog_instances {
            let class_link = if row.class_ref.trim().is_empty() {
                "-".to_string()
            } else {
                format!("[{}](#class-{})", row.class_ref, slugify_anchor(&row.class_ref))
            };
            let preset_name = if row.validation_preset.trim().is_empty() {
                "-".to_string()
            } else {
                row.validation_preset.clone()
            };
            let props = if row.properties.is_empty() {
                "-".to_string()
            } else {
                let mut keys: Vec<&String> = row.properties.keys().collect();
                keys.sort();
                keys.iter()
                    .map(|k| format!("{}={}", k, row.properties.get(*k).map(|v| v.as_str()).unwrap_or("")))
                    .collect::<Vec<String>>()
                    .join(", ")
            };
            lines.push(format!(
                "- [{}] {} - {} (class: {}, preset: {}, attrs: {})",
                row.catalog, row.code, row.label, class_link, preset_name, props
            ));
        }
    }

    lines.join("\n")
}

fn slugify_anchor(value: &str) -> String {
    let mut out = String::new();
    let mut prev_dash = false;
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            prev_dash = false;
        } else if !prev_dash {
            out.push('-');
            prev_dash = true;
        }
    }

    while out.starts_with('-') {
        out.remove(0);
    }
    while out.ends_with('-') {
        out.pop();
    }

    if out.is_empty() {
        "item".to_string()
    } else {
        out
    }
}

fn validate_class(class: &Class, issues: &mut Vec<ValidationIssue>) {
    let class_path = format!("class:{}", class.name);

    if !is_identifier(&class.name) {
        issues.push(error_issue_at(
            "CLASS_NAME_INVALID",
            &format!("Class '{}' is invalid: INTERLIS identifiers must start with a letter and then contain only A-Z, a-z, 0-9 or '_' (no spaces)", class.name),
            Some(class_path.clone()),
        ));
    }

    if class.attributes.is_empty() {
        issues.push(warn_issue_at(
            "CLASS_WITHOUT_ATTRIBUTES",
            &format!("Class '{}' has no attributes", class.name),
            Some(class_path.clone()),
        ));
    }

    let mut seen = HashSet::new();

    for attr in &class.attributes {
        let attr_path = format!("attr:{}:{}", class.name, attr.name);

        if !is_identifier(&attr.name) {
            issues.push(error_issue_at(
                "ATTRIBUTE_NAME_INVALID",
                &format!(
                    "Attribute '{}.{}' is invalid: INTERLIS identifiers must start with a letter and then contain only A-Z, a-z, 0-9 or '_' (no spaces)",
                    class.name, attr.name
                ),
                Some(attr_path.clone()),
            ));
        }

        if attr.r#type.trim().is_empty() {
            issues.push(error_issue_at(
                "ATTRIBUTE_TYPE_EMPTY",
                &format!(
                    "Attribute '{}.{}' has empty type; provide a valid INTERLIS type expression",
                    class.name, attr.name
                ),
                Some(attr_path.clone()),
            ));
        }

        if !seen.insert(attr.name.as_str()) {
            issues.push(error_issue_at(
                "ATTRIBUTE_DUPLICATE_IN_CLASS",
                &format!(
                    "Attribute '{}.{}' is duplicated in class '{}'",
                    class.name, attr.name, class.name
                ),
                Some(attr_path),
            ));
        }
    }
}

fn validate_domains(domains: &[Domain], issues: &mut Vec<ValidationIssue>) {
    for domain in domains {
        let path = format!("domain:{}", domain.name);
        if !is_identifier(&domain.name) {
            issues.push(error_issue_at(
                "DOMAIN_NAME_INVALID",
                &format!("Domain '{}' is invalid: INTERLIS identifiers must start with a letter and then contain only A-Z, a-z, 0-9 or '_' (no spaces)", domain.name),
                Some(path.clone()),
            ));
        }
        if domain.values.is_empty() && domain.r#type.trim().is_empty() {
            issues.push(error_issue_at(
                "DOMAIN_TYPE_EMPTY",
                &format!(
                    "Domain '{}' has no value list and empty type; define values or provide a valid INTERLIS type expression",
                    domain.name
                ),
                Some(path.clone()),
            ));
        }

        if !domain.values.is_empty() {
            let mut seen = HashSet::new();
            for value in &domain.values {
                let trimmed = value.trim();
                if trimmed.is_empty() {
                    issues.push(error_issue_at(
                        "DOMAIN_VALUE_EMPTY",
                        &format!("Domain '{}' contains an empty value", domain.name),
                        Some(path.clone()),
                    ));
                    continue;
                }
                let key = trimmed.to_ascii_lowercase();
                if !seen.insert(key) {
                    issues.push(error_issue_at(
                        "DOMAIN_VALUE_DUPLICATE",
                        &format!("Domain '{}' contains duplicate value '{}'", domain.name, trimmed),
                        Some(format!("domainval:{}:{}", domain.name, trimmed)),
                    ));
                }
            }
        }
    }
}

fn validate_associations(associations: &[Association], classes: &[Class], issues: &mut Vec<ValidationIssue>) {
    let class_names: HashSet<&str> = classes.iter().map(|c| c.name.as_str()).collect();

    for assoc in associations {
        let path = format!("association:{}", assoc.name);
        if !is_identifier(&assoc.name) {
            issues.push(error_issue_at(
                "ASSOCIATION_NAME_INVALID",
                &format!("Association '{}' is invalid: INTERLIS identifiers must start with a letter and then contain only A-Z, a-z, 0-9 or '_' (no spaces)", assoc.name),
                Some(path.clone()),
            ));
        }
        if !class_names.contains(assoc.from_class.as_str()) {
            issues.push(error_issue_at(
                "ASSOCIATION_FROM_CLASS_UNKNOWN",
                &format!("Association '{}' refers to unknown from_class '{}'", assoc.name, assoc.from_class),
                Some(path.clone()),
            ));
        }
        if !class_names.contains(assoc.to_class.as_str()) {
            issues.push(error_issue_at(
                "ASSOCIATION_TO_CLASS_UNKNOWN",
                &format!("Association '{}' refers to unknown to_class '{}'", assoc.name, assoc.to_class),
                Some(path.clone()),
            ));
        }
        if !is_cardinality(&assoc.cardinality) {
            issues.push(error_issue_at(
                "ASSOCIATION_CARDINALITY_INVALID",
                &format!("Association '{}' has invalid cardinality '{}'; use min..max (e.g. 0..1, 1..*, 0..*)", assoc.name, assoc.cardinality),
                Some(path),
            ));
        }
    }
}

fn validate_inheritance(classes: &[Class], issues: &mut Vec<ValidationIssue>) {
    let class_names: HashSet<&str> = classes.iter().map(|c| c.name.as_str()).collect();
    let extends_by_class: HashMap<&str, &str> = classes
        .iter()
        .map(|c| (c.name.as_str(), c.extends.trim()))
        .collect();

    for class in classes {
        let extends = class.extends.trim();
        if extends.is_empty() {
            continue;
        }

        let path = format!("class:{}", class.name);
        if !is_identifier(extends) {
            issues.push(error_issue_at(
                "CLASS_EXTENDS_INVALID",
                &format!(
                    "Class '{}' has invalid EXTENDS '{}': INTERLIS identifiers must start with a letter and then contain only A-Z, a-z, 0-9 or '_' (no spaces)",
                    class.name, extends
                ),
                Some(path.clone()),
            ));
            continue;
        }

        if extends == class.name {
            issues.push(error_issue_at(
                "CLASS_EXTENDS_SELF",
                &format!("Class '{}' cannot EXTEND itself", class.name),
                Some(path.clone()),
            ));
            continue;
        }

        if !class_names.contains(extends) {
            issues.push(error_issue_at(
                "CLASS_EXTENDS_UNKNOWN",
                &format!("Class '{}' EXTENDS unknown class '{}'", class.name, extends),
                Some(path.clone()),
            ));
            continue;
        }

        let mut seen = HashSet::new();
        let mut current = class.name.as_str();
        while let Some(parent) = extends_by_class.get(current) {
            let parent = parent.trim();
            if parent.is_empty() {
                break;
            }
            if !seen.insert(parent.to_string()) {
                issues.push(error_issue_at(
                    "CLASS_EXTENDS_CYCLE",
                    &format!("Inheritance cycle detected involving class '{}'", class.name),
                    Some(path.clone()),
                ));
                break;
            }
            current = parent;
        }
    }
}

fn validate_topics(topics: &[Topic], issues: &mut Vec<ValidationIssue>) {
    let mut seen_topic_names = HashSet::new();
    for topic in topics {
        let topic_path = format!("topic:{}", topic.name);
        if !is_identifier(&topic.name) {
            issues.push(error_issue_at(
                "TOPIC_NAME_INVALID",
                &format!("Topic '{}' is invalid: INTERLIS identifiers must start with a letter and then contain only A-Z, a-z, 0-9 or '_' (no spaces)", topic.name),
                Some(topic_path.clone()),
            ));
        }

        if !seen_topic_names.insert(topic.name.as_str()) {
            issues.push(error_issue_at(
                "TOPIC_NAME_DUPLICATE",
                &format!("Topic '{}' appears multiple times", topic.name),
                Some(topic_path.clone()),
            ));
        }

        let mut seen_subtopic_names = HashSet::new();
        for subtopic in &topic.subtopics {
            let subtopic_path = format!("subtopic:{}:{}", topic.name, subtopic.name);
            if !is_identifier(&subtopic.name) {
                issues.push(error_issue_at(
                    "SUBTOPIC_NAME_INVALID",
                    &format!("Subtopic '{}.{}' is invalid: INTERLIS identifiers must start with a letter and then contain only A-Z, a-z, 0-9 or '_' (no spaces)", topic.name, subtopic.name),
                    Some(subtopic_path.clone()),
                ));
            }
            if !seen_subtopic_names.insert(subtopic.name.as_str()) {
                issues.push(error_issue_at(
                    "SUBTOPIC_NAME_DUPLICATE",
                    &format!("Subtopic '{}.{}' appears multiple times", topic.name, subtopic.name),
                    Some(subtopic_path),
                ));
            }
        }
    }
}

fn validate_duplicates(
    classes: &[Class],
    domains: &[Domain],
    associations: &[Association],
    issues: &mut Vec<ValidationIssue>,
) {
    let mut seen_class = HashSet::new();
    for class in classes {
        if !seen_class.insert(class.name.as_str()) {
            issues.push(error_issue_at(
                "CLASS_NAME_DUPLICATE",
                &format!("Class '{}' appears multiple times", class.name),
                Some(format!("class:{}", class.name)),
            ));
        }
    }

    let mut seen_domain = HashSet::new();
    for domain in domains {
        if !seen_domain.insert(domain.name.as_str()) {
            issues.push(error_issue_at(
                "DOMAIN_NAME_DUPLICATE",
                &format!("Domain '{}' appears multiple times", domain.name),
                Some(format!("domain:{}", domain.name)),
            ));
        }
    }

    let mut seen_assoc = HashSet::new();
    for assoc in associations {
        if !seen_assoc.insert(assoc.name.as_str()) {
            issues.push(error_issue_at(
                "ASSOCIATION_NAME_DUPLICATE",
                &format!("Association '{}' appears multiple times", assoc.name),
                Some(format!("association:{}", assoc.name)),
            ));
        }
    }

}

fn is_cardinality(value: &str) -> bool {
    let trimmed = value.trim();
    if let Some((min, max)) = trimmed.split_once("..") {
        let ok_min = min == "*" || min.parse::<u32>().is_ok();
        let ok_max = max == "*" || max.parse::<u32>().is_ok();
        return ok_min && ok_max;
    }
    false
}

fn error_issue(code: &'static str, message: &str) -> ValidationIssue {
    error_issue_at(code, message, None)
}

fn info_issue(code: &'static str, message: &str) -> ValidationIssue {
    ValidationIssue {
        level: "info",
        code,
        message: message.to_string(),
        node_path: None,
        line: None,
        column: None,
    }
}

fn info_issue_at(code: &'static str, message: &str, node_path: Option<String>) -> ValidationIssue {
    ValidationIssue {
        level: "info",
        code,
        message: message.to_string(),
        node_path,
        line: None,
        column: None,
    }
}

fn error_issue_at(code: &'static str, message: &str, node_path: Option<String>) -> ValidationIssue {
    ValidationIssue {
        level: "error",
        code,
        message: message.to_string(),
        node_path,
        line: None,
        column: None,
    }
}

fn warn_issue(code: &'static str, message: &str) -> ValidationIssue {
    warn_issue_at(code, message, None)
}

fn warn_issue_at(code: &'static str, message: &str, node_path: Option<String>) -> ValidationIssue {
    ValidationIssue {
        level: "warning",
        code,
        message: message.to_string(),
        node_path,
        line: None,
        column: None,
    }
}

fn is_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    match chars.next() {
        Some(ch) if ch.is_ascii_alphabetic() => {}
        _ => return false,
    }

    chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

fn is_numeric_literal(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty() && trimmed.parse::<f64>().is_ok()
}

fn catalog_instance_attr_value<'a>(row: &'a CatalogInstance, attr_name: &str) -> Option<&'a str> {
    if attr_name.eq_ignore_ascii_case("code") {
        return Some(row.code.as_str());
    }
    if attr_name.eq_ignore_ascii_case("label") {
        return Some(row.label.as_str());
    }

    for (key, value) in &row.properties {
        if key.eq_ignore_ascii_case(attr_name) {
            return Some(value.as_str());
        }
    }
    None
}

fn value_matches_type(value: &str, type_name: &str) -> bool {
    let t = type_name.trim().to_ascii_uppercase();
    let v = value.trim();
    if v.is_empty() {
        return true;
    }

    match t.as_str() {
        "TEXT" => true,
        "BOOLEAN" => {
            v.eq_ignore_ascii_case("true")
                || v.eq_ignore_ascii_case("false")
                || v.eq_ignore_ascii_case("yes")
                || v.eq_ignore_ascii_case("no")
                || v == "1"
                || v == "0"
        }
        "NUMERIC" => is_numeric_literal(v),
        "DATE" => is_iso_like_date(v),
        "TIMEOFDAY" => is_iso_like_time(v),
        "DATETIME" => is_iso_like_datetime(v),
        "UUID" => is_uuid_like(v),
        _ => true,
    }
}

fn is_iso_like_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 10 {
        return false;
    }
    bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(i, b)| i == 4 || i == 7 || b.is_ascii_digit())
}

fn is_iso_like_time(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 8 {
        return false;
    }
    bytes[2] == b':'
        && bytes[5] == b':'
        && bytes
            .iter()
            .enumerate()
            .all(|(i, b)| i == 2 || i == 5 || b.is_ascii_digit())
}

fn is_iso_like_datetime(value: &str) -> bool {
    if let Some((date, time)) = value.split_once('T') {
        if time.ends_with('Z') {
            return is_iso_like_date(date) && is_iso_like_time(time.trim_end_matches('Z'));
        }
        return is_iso_like_date(date) && is_iso_like_time(time);
    }
    false
}

fn is_valid_preset_actor(value: &str) -> bool {
    let len = value.len();
    if !(3..=64).contains(&len) {
        return false;
    }
    value
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'.' || b == b'@' || b == b'-')
}

fn is_uuid_like(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 36 {
        return false;
    }
    for (i, b) in bytes.iter().enumerate() {
        let ok_sep = (i == 8 || i == 13 || i == 18 || i == 23) && *b == b'-';
        let ok_hex = b.is_ascii_hexdigit();
        if !(ok_sep || ok_hex) {
            return false;
        }
    }
    true
}
