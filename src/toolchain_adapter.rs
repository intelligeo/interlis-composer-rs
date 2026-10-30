use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug)]
pub struct CommandSpec {
    pub enabled: bool,
    pub bin: String,
    pub args: Vec<String>,
    pub strict: bool,
}

#[derive(Clone, Debug)]
pub struct ToolchainConfig {
    pub ili2c: CommandSpec,
    pub ilivalidator: CommandSpec,
    pub iox_ili: CommandSpec,
    pub model_repositories: Vec<String>,
    pub proxy: ProxyConfig,
    pub timeout_ms: u64,
}

#[derive(Clone, Debug, Default)]
pub struct ProxyConfig {
    pub http_proxy: Option<String>,
    pub https_proxy: Option<String>,
    pub no_proxy: Option<String>,
    pub java_opts: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum ToolCommand {
    ValidateModel,
    ValidateTransfer,
    ExportTransfer,
}

#[derive(Debug, Clone)]
pub struct ToolRequest {
    pub request_id: String,
    pub model_name: String,
    pub interlis_text: String,
    pub transfer_text: Option<String>,
    pub transfer_format: Option<String>,
    pub instances_json: Option<String>,
    pub timeout_ms: Option<u64>,
    pub strict: bool,
}

#[derive(Debug, Clone)]
pub struct ToolIssue {
    pub severity: &'static str,
    pub code: &'static str,
    pub message: String,
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub path: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ToolResponse {
    pub ok: bool,
    pub issues: Vec<ToolIssue>,
    pub output_file_name: Option<String>,
    pub output_content: Option<String>,
    pub duration_ms: u64,
}

#[derive(Debug)]
pub enum ToolExecError {
    Io(String),
    Timeout,
    Process(String),
}

#[derive(Debug)]
struct CommandRunResult {
    success: bool,
    exit_code: Option<i32>,
    output: String,
}

pub fn load_config() -> ToolchainConfig {
    let ili2c_bin = env::var("ILI2C_BIN").unwrap_or_else(|_| "ili2c".to_string());
    let ili2c_args = parse_env_args("ILI2C_ARGS");
    let ilivalidator_bin = env::var("ILIVALIDATOR_BIN").unwrap_or_else(|_| "ilivalidator".to_string());
    let ilivalidator_args = parse_env_args("ILIVALIDATOR_ARGS");
    let iox_ili_bin = env::var("IOX_ILI_BIN").unwrap_or_else(|_| "iox-ili".to_string());
    let iox_ili_args = parse_env_args("IOX_ILI_ARGS");
    let model_repositories = parse_model_repositories();
    let proxy = parse_proxy_config();

    ToolchainConfig {
        ili2c: CommandSpec {
            enabled: parse_bool_env("ILI2C_ENABLED", true),
            bin: ili2c_bin,
            args: ili2c_args,
            strict: parse_bool_env("ILI2C_STRICT", false),
        },
        ilivalidator: CommandSpec {
            enabled: parse_bool_env("ILIVALIDATOR_ENABLED", true),
            bin: ilivalidator_bin,
            args: ilivalidator_args,
            strict: parse_bool_env("ILIVALIDATOR_STRICT", false),
        },
        iox_ili: CommandSpec {
            enabled: parse_bool_env("IOX_ILI_ENABLED", false),
            bin: iox_ili_bin,
            args: iox_ili_args,
            strict: parse_bool_env("IOX_ILI_STRICT", false),
        },
        model_repositories,
        proxy,
        timeout_ms: parse_u64_env("IMC_TOOLCHAIN_TIMEOUT_MS", 30_000),
    }
}

pub fn execute(
    config: &ToolchainConfig,
    cmd: ToolCommand,
    req: &ToolRequest,
) -> Result<ToolResponse, ToolExecError> {
    let started = Instant::now();
    let timeout_ms = req.timeout_ms.unwrap_or(config.timeout_ms);

    let response = match cmd {
        ToolCommand::ValidateModel => execute_validate_model(config, req, timeout_ms),
        ToolCommand::ValidateTransfer => execute_validate_transfer(config, req, timeout_ms),
        ToolCommand::ExportTransfer => execute_export_transfer(config, req, timeout_ms),
    };

    match response {
        Ok(mut resp) => {
            resp.duration_ms = started.elapsed().as_millis() as u64;
            Ok(resp)
        }
        Err(err) => Err(err),
    }
}

fn execute_validate_model(
    config: &ToolchainConfig,
    req: &ToolRequest,
    timeout_ms: u64,
) -> Result<ToolResponse, ToolExecError> {
    if !config.ili2c.enabled {
        return Ok(ToolResponse {
            ok: true,
            issues: vec![tool_issue(
                "warning",
                "ILI2C_DISABLED",
                "ili2c integration is disabled by configuration",
                None,
                None,
                Some(format!("model:{}", req.model_name)),
            )],
            output_file_name: None,
            output_content: None,
            duration_ms: 0,
        });
    }

    let workdir = create_temp_dir("imc-ili2c")?;
    let model_path = workdir.join("model.ili");
    fs::write(&model_path, req.interlis_text.as_bytes())
        .map_err(|e| ToolExecError::Io(format!("unable to write model.ili: {}", e)))?;

    let mut args = config.ili2c.args.clone();
    append_modeldir_args_if_missing(&mut args, &workdir, &config.model_repositories);
    args.push(model_path.to_string_lossy().to_string());

    match run_command_with_timeout(config, &config.ili2c, &args, Some(&workdir), timeout_ms) {
        Ok(run) => {
            let mut issues = parse_output("ILI2C", &run.output, if run.success { "warning" } else { "error" });
            if issues.is_empty() {
                if run.success {
                    issues.push(tool_issue(
                        "info",
                        "ILI2C_OK",
                        "ili2c completed without diagnostics",
                        None,
                        None,
                        Some(format!("model:{}", req.model_name)),
                    ));
                } else {
                    issues.push(tool_issue(
                        "error",
                        "ILI2C_FAILED",
                        &format!("ili2c failed with exit code {:?}", run.exit_code),
                        None,
                        None,
                        Some(format!("model:{}", req.model_name)),
                    ));
                }
            }

            Ok(ToolResponse {
                ok: run.success,
                issues,
                output_file_name: None,
                output_content: None,
                duration_ms: 0,
            })
        }
        Err(ToolExecError::Process(message)) => Ok(ToolResponse {
            ok: !config.ili2c.strict,
            issues: vec![tool_issue(
                if config.ili2c.strict || req.strict { "error" } else { "warning" },
                "TOOL_UNAVAILABLE",
                &format!("unable to execute ili2c: {}", message),
                None,
                None,
                Some(format!("model:{}", req.model_name)),
            )],
            output_file_name: None,
            output_content: None,
            duration_ms: 0,
        }),
        Err(err) => Err(err),
    }
}

fn execute_validate_transfer(
    config: &ToolchainConfig,
    req: &ToolRequest,
    timeout_ms: u64,
) -> Result<ToolResponse, ToolExecError> {
    let transfer = req.transfer_text.clone().unwrap_or_default();
    if transfer.trim().is_empty() {
        return Ok(ToolResponse {
            ok: false,
            issues: vec![tool_issue(
                "error",
                "TRANSFER_INPUT_EMPTY",
                "transfer input is empty",
                None,
                None,
                Some("xtf".to_string()),
            )],
            output_file_name: None,
            output_content: None,
            duration_ms: 0,
        });
    }

    if !config.ilivalidator.enabled {
        return Ok(ToolResponse {
            ok: true,
            issues: vec![tool_issue(
                "warning",
                "ILIVALIDATOR_DISABLED",
                "ilivalidator integration is disabled by configuration",
                None,
                None,
                Some("xtf".to_string()),
            )],
            output_file_name: None,
            output_content: None,
            duration_ms: 0,
        });
    }

    let workdir = create_temp_dir("imc-ilivalidator")?;
    let model_path = workdir.join("model.ili");
    let transfer_ext = match req.transfer_format.as_deref() {
        Some("xml") => "xml",
        _ => "xtf",
    };
    let transfer_path = workdir.join(format!("transfer.{}", transfer_ext));

    fs::write(&model_path, req.interlis_text.as_bytes())
        .and_then(|_| fs::write(&transfer_path, transfer.as_bytes()))
        .map_err(|e| ToolExecError::Io(format!("unable to prepare transfer validation input: {}", e)))?;

    let mut args = config.ilivalidator.args.clone();
    append_modeldir_args_if_missing(&mut args, &workdir, &config.model_repositories);
    args.push("--models".to_string());
    args.push(req.model_name.clone());
    args.push(transfer_path.to_string_lossy().to_string());

    match run_command_with_timeout(config, &config.ilivalidator, &args, Some(&workdir), timeout_ms) {
        Ok(run) => {
            let mut issues = parse_output(
                "ILIVALIDATOR",
                &run.output,
                if run.success { "warning" } else { "error" },
            );
            if issues.is_empty() {
                if run.success {
                    issues.push(tool_issue(
                        "info",
                        "ILIVALIDATOR_OK",
                        "ilivalidator completed without diagnostics",
                        None,
                        None,
                        Some("xtf".to_string()),
                    ));
                } else {
                    issues.push(tool_issue(
                        "error",
                        "ILIVALIDATOR_FAILED",
                        &format!("ilivalidator failed with exit code {:?}", run.exit_code),
                        None,
                        None,
                        Some("xtf".to_string()),
                    ));
                }
            }

            Ok(ToolResponse {
                ok: run.success,
                issues,
                output_file_name: None,
                output_content: None,
                duration_ms: 0,
            })
        }
        Err(ToolExecError::Process(message)) => Ok(ToolResponse {
            ok: !config.ilivalidator.strict,
            issues: vec![tool_issue(
                if config.ilivalidator.strict || req.strict {
                    "error"
                } else {
                    "warning"
                },
                "TOOL_UNAVAILABLE",
                &format!("unable to execute ilivalidator: {}", message),
                None,
                None,
                Some("xtf".to_string()),
            )],
            output_file_name: None,
            output_content: None,
            duration_ms: 0,
        }),
        Err(err) => Err(err),
    }
}

fn execute_export_transfer(
    config: &ToolchainConfig,
    req: &ToolRequest,
    timeout_ms: u64,
) -> Result<ToolResponse, ToolExecError> {
    if !config.iox_ili.enabled {
        return Ok(ToolResponse {
            ok: false,
            issues: vec![tool_issue(
                "warning",
                "XTF_EXPORT_NOT_CONFIGURED",
                "iox-ili export wrapper is not configured (set IOX_ILI_ENABLED=true)",
                None,
                None,
                Some("xtf".to_string()),
            )],
            output_file_name: None,
            output_content: None,
            duration_ms: 0,
        });
    }

    let workdir = create_temp_dir("imc-iox-ili")?;
    let model_path = workdir.join("model.ili");
    let instances_path = workdir.join("instances.json");
    let output_path = workdir.join("export.xtf");

    fs::write(&model_path, req.interlis_text.as_bytes())
        .and_then(|_| {
            fs::write(
                &instances_path,
                req.instances_json.clone().unwrap_or_else(|| "[]".to_string()).as_bytes(),
            )
        })
        .map_err(|e| ToolExecError::Io(format!("unable to prepare export input: {}", e)))?;

    let mut args = config.iox_ili.args.clone();
    args.push("export-transfer".to_string());
    args.push("--model".to_string());
    args.push(model_path.to_string_lossy().to_string());
    args.push("--instances".to_string());
    args.push(instances_path.to_string_lossy().to_string());
    args.push("--output".to_string());
    args.push(output_path.to_string_lossy().to_string());
    args.push("--model-name".to_string());
    args.push(req.model_name.clone());

    match run_command_with_timeout(config, &config.iox_ili, &args, Some(&workdir), timeout_ms) {
        Ok(run) => {
            let mut issues = parse_output("IOX_ILI", &run.output, if run.success { "warning" } else { "error" });
            if !run.success {
                if issues.is_empty() {
                    issues.push(tool_issue(
                        "error",
                        "EXPORT_FAILED",
                        &format!("iox-ili export failed with exit code {:?}", run.exit_code),
                        None,
                        None,
                        Some("xtf".to_string()),
                    ));
                }
                return Ok(ToolResponse {
                    ok: false,
                    issues,
                    output_file_name: None,
                    output_content: None,
                    duration_ms: 0,
                });
            }

            let content = fs::read_to_string(&output_path)
                .map_err(|e| ToolExecError::Io(format!("unable to read exported transfer: {}", e)))?;
            if issues.is_empty() {
                issues.push(tool_issue(
                    "info",
                    "EXPORT_OK",
                    "transfer export completed",
                    None,
                    None,
                    Some("xtf".to_string()),
                ));
            }

            Ok(ToolResponse {
                ok: true,
                issues,
                output_file_name: Some(format!("{}.xtf", sanitize_file_stem(&req.model_name))),
                output_content: Some(content),
                duration_ms: 0,
            })
        }
        Err(ToolExecError::Process(message)) => Ok(ToolResponse {
            ok: false,
            issues: vec![tool_issue(
                if config.iox_ili.strict || req.strict { "error" } else { "warning" },
                "TOOL_UNAVAILABLE",
                &format!("unable to execute iox-ili export wrapper: {}", message),
                None,
                None,
                Some("xtf".to_string()),
            )],
            output_file_name: None,
            output_content: None,
            duration_ms: 0,
        }),
        Err(err) => Err(err),
    }
}

fn run_command_with_timeout(
    config: &ToolchainConfig,
    spec: &CommandSpec,
    args: &[String],
    cwd: Option<&Path>,
    timeout_ms: u64,
) -> Result<CommandRunResult, ToolExecError> {
    let mut cmd = Command::new(&spec.bin);
    cmd.args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::null());

    if let Some(base) = cwd {
        cmd.current_dir(base);
    }

    apply_proxy_environment(&mut cmd, config, spec);

    let mut child = cmd
        .spawn()
        .map_err(|e| ToolExecError::Process(e.to_string()))?;

    let deadline = Instant::now() + Duration::from_millis(timeout_ms.max(1));
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let output = child
                    .wait_with_output()
                    .map_err(|e| ToolExecError::Io(format!("failed to read process output: {}", e)))?;

                let mut merged = String::new();
                if !output.stdout.is_empty() {
                    merged.push_str(&String::from_utf8_lossy(&output.stdout));
                }
                if !output.stderr.is_empty() {
                    if !merged.is_empty() {
                        merged.push('\n');
                    }
                    merged.push_str(&String::from_utf8_lossy(&output.stderr));
                }

                return Ok(CommandRunResult {
                    success: status.success(),
                    exit_code: status.code(),
                    output: merged,
                });
            }
            Ok(None) => {
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(ToolExecError::Timeout);
                }
                thread::sleep(Duration::from_millis(30));
            }
            Err(err) => return Err(ToolExecError::Io(format!("failed waiting process: {}", err))),
        }
    }
}

fn parse_proxy_config() -> ProxyConfig {
    let explicit_http = first_non_empty_env(&["IMC_HTTP_PROXY", "HTTP_PROXY", "http_proxy"]);
    let explicit_https = first_non_empty_env(&["IMC_HTTPS_PROXY", "HTTPS_PROXY", "https_proxy"]);
    let fallback = first_non_empty_env(&["IMC_PROXY", "ALL_PROXY", "all_proxy"]);

    let http_proxy = explicit_http.or_else(|| fallback.clone());
    let https_proxy = explicit_https.or_else(|| http_proxy.clone()).or(fallback);
    let no_proxy = first_non_empty_env(&["IMC_NO_PROXY", "NO_PROXY", "no_proxy"]);

    let java_opts = build_java_proxy_opts(http_proxy.as_deref(), https_proxy.as_deref(), no_proxy.as_deref());

    ProxyConfig {
        http_proxy,
        https_proxy,
        no_proxy,
        java_opts,
    }
}

fn first_non_empty_env(keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|key| env::var(key).ok().map(|v| v.trim().to_string()))
        .filter(|value| !value.is_empty())
}

fn apply_proxy_environment(cmd: &mut Command, config: &ToolchainConfig, spec: &CommandSpec) {
    if let Some(value) = &config.proxy.http_proxy {
        cmd.env("HTTP_PROXY", value);
        cmd.env("http_proxy", value);
    }
    if let Some(value) = &config.proxy.https_proxy {
        cmd.env("HTTPS_PROXY", value);
        cmd.env("https_proxy", value);
    }
    if let Some(value) = &config.proxy.no_proxy {
        cmd.env("NO_PROXY", value);
        cmd.env("no_proxy", value);
    }

    if is_java_command(spec) {
        let merged = merge_java_tool_options(
            env::var("JAVA_TOOL_OPTIONS").ok().as_deref(),
            config.proxy.java_opts.as_deref(),
        );
        if let Some(value) = merged {
            cmd.env("JAVA_TOOL_OPTIONS", value);
        }
    }
}

fn is_java_command(spec: &CommandSpec) -> bool {
    let bin_name = spec
        .bin
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(spec.bin.as_str())
        .to_ascii_lowercase();
    bin_name == "java" || bin_name == "java.exe"
}

fn merge_java_tool_options(current: Option<&str>, extra: Option<&str>) -> Option<String> {
    match (current.map(str::trim).filter(|v| !v.is_empty()), extra.map(str::trim).filter(|v| !v.is_empty())) {
        (Some(base), Some(add)) => Some(format!("{} {}", base, add)),
        (Some(base), None) => Some(base.to_string()),
        (None, Some(add)) => Some(add.to_string()),
        (None, None) => None,
    }
}

fn build_java_proxy_opts(http_proxy: Option<&str>, https_proxy: Option<&str>, no_proxy: Option<&str>) -> Option<String> {
    let mut opts: Vec<String> = Vec::new();

    if let Some(url) = http_proxy.and_then(parse_proxy_url) {
        opts.push(format!("-Dhttp.proxyHost={}", url.host));
        if let Some(port) = url.port {
            opts.push(format!("-Dhttp.proxyPort={}", port));
        }
    }

    if let Some(url) = https_proxy.and_then(parse_proxy_url) {
        opts.push(format!("-Dhttps.proxyHost={}", url.host));
        if let Some(port) = url.port {
            opts.push(format!("-Dhttps.proxyPort={}", port));
        }
    }

    if let Some(raw_no_proxy) = no_proxy {
        let hosts = normalize_no_proxy_for_java(raw_no_proxy);
        if !hosts.is_empty() {
            opts.push(format!("-Dhttp.nonProxyHosts={}", hosts));
            opts.push(format!("-Dhttps.nonProxyHosts={}", hosts));
        }
    }

    if opts.is_empty() {
        None
    } else {
        Some(opts.join(" "))
    }
}

#[derive(Debug)]
struct ParsedProxyUrl {
    host: String,
    port: Option<u16>,
}

fn parse_proxy_url(raw: &str) -> Option<ParsedProxyUrl> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    let no_scheme = trimmed
        .strip_prefix("http://")
        .or_else(|| trimmed.strip_prefix("https://"))
        .unwrap_or(trimmed);
    let no_auth = no_scheme.rsplit('@').next().unwrap_or(no_scheme);
    let host_port = no_auth.split('/').next().unwrap_or(no_auth).trim();
    if host_port.is_empty() {
        return None;
    }

    if let Some((host, port)) = host_port.rsplit_once(':') {
        if let Ok(port_num) = port.parse::<u16>() {
            let normalized_host = host.trim().trim_matches('[').trim_matches(']').to_string();
            if !normalized_host.is_empty() {
                return Some(ParsedProxyUrl {
                    host: normalized_host,
                    port: Some(port_num),
                });
            }
        }
    }

    Some(ParsedProxyUrl {
        host: host_port.trim_matches('[').trim_matches(']').to_string(),
        port: None,
    })
}

fn normalize_no_proxy_for_java(raw: &str) -> String {
    raw.split(|ch| ch == ',' || ch == ';' || ch == ' ')
        .filter_map(|token| {
            let value = token.trim();
            if value.is_empty() {
                return None;
            }
            let no_port = value.split(':').next().unwrap_or(value).trim();
            if no_port.is_empty() {
                return None;
            }
            let java_pattern = if no_port == "*" {
                "*".to_string()
            } else if let Some(host) = no_port.strip_prefix("*.") {
                format!("*.{}", host)
            } else if no_port.starts_with('.') {
                format!("*{}", no_port)
            } else {
                no_port.to_string()
            };
            Some(java_pattern)
        })
        .collect::<Vec<String>>()
        .join("|")
}

fn parse_output(tool_prefix: &str, output: &str, default_level: &'static str) -> Vec<ToolIssue> {
    let mut issues = Vec::new();

    for raw_line in output.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }

        let lower = line.to_ascii_lowercase();
        let severity = if lower.contains("error") {
            "error"
        } else if lower.contains("warn") || lower.contains("warning") {
            "warning"
        } else if lower.contains("info") {
            "info"
        } else {
            default_level
        };

        let code = match (tool_prefix, severity) {
            ("ILI2C", "error") => "ILI_SYNTAX_ERROR",
            ("ILI2C", "warning") => "ILI_SEMANTIC_WARNING",
            ("ILIVALIDATOR", "error") => "TRANSFER_VALIDATION_ERROR",
            ("ILIVALIDATOR", "warning") => "TRANSFER_VALIDATION_WARNING",
            ("IOX_ILI", "error") => "EXPORT_ERROR",
            ("IOX_ILI", "warning") => "EXPORT_WARNING",
            (_, "error") => "TOOLCHAIN_GENERIC_ERROR",
            (_, "warning") => "TOOLCHAIN_GENERIC_WARNING",
            _ => "TOOLCHAIN_INFO",
        };

        let (line_no, col_no) = extract_line_column(line);
        issues.push(tool_issue(severity, code, line, line_no, col_no, None));
    }

    issues
}

fn extract_line_column(text: &str) -> (Option<usize>, Option<usize>) {
    let tokens: Vec<&str> = text
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|tok| !tok.is_empty())
        .collect();

    let mut line = None;
    let mut col = None;

    for idx in 0..tokens.len() {
        let current = tokens[idx].to_ascii_lowercase();
        if line.is_none() && (current == "line" || current == "ln") {
            if let Some(next) = tokens.get(idx + 1).and_then(|v| v.parse::<usize>().ok()) {
                line = Some(next);
            }
        }

        if col.is_none() && (current == "column" || current == "col") {
            if let Some(next) = tokens.get(idx + 1).and_then(|v| v.parse::<usize>().ok()) {
                col = Some(next);
            }
        }
    }

    (line, col)
}

fn tool_issue(
    severity: &'static str,
    code: &'static str,
    message: &str,
    line: Option<usize>,
    column: Option<usize>,
    path: Option<String>,
) -> ToolIssue {
    ToolIssue {
        severity,
        code,
        message: message.to_string(),
        line,
        column,
        path,
    }
}

fn create_temp_dir(prefix: &str) -> Result<PathBuf, ToolExecError> {
    let base = env::temp_dir();
    let pid = std::process::id();
    let now_ns = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);

    for seq in 0..16_u32 {
        let dir = base.join(format!("{}-{}-{}-{}", prefix, now_ns, pid, seq));
        if fs::create_dir_all(&dir).is_ok() {
            return Ok(dir);
        }
    }

    Err(ToolExecError::Io("unable to create temporary working directory".to_string()))
}

fn parse_bool_env(key: &str, default_value: bool) -> bool {
    match env::var(key) {
        Ok(raw) => {
            let value = raw.trim().to_ascii_lowercase();
            matches!(value.as_str(), "1" | "true" | "yes" | "on")
        }
        Err(_) => default_value,
    }
}

fn parse_u64_env(key: &str, default_value: u64) -> u64 {
    env::var(key)
        .ok()
        .and_then(|v| v.trim().parse::<u64>().ok())
        .unwrap_or(default_value)
}

fn parse_env_args(key: &str) -> Vec<String> {
    env::var(key)
        .ok()
        .map(|v| {
            v.split_whitespace()
                .filter(|s| !s.trim().is_empty())
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
        })
        .unwrap_or_default()
}

fn parse_model_repositories() -> Vec<String> {
    let raw = env::var("IMC_MODEL_REPOSITORIES")
        .unwrap_or_else(|_| "https://models.interlis.ch/;https://models.geo.admin.ch/".to_string());

    let mut repos = Vec::new();
    for token in raw.split(|ch| ch == ';' || ch == ',' || ch == '\n' || ch == '\r') {
        let value = token.trim();
        if value.is_empty() {
            continue;
        }
        if !repos.iter().any(|existing| existing == value) {
            repos.push(value.to_string());
        }
    }
    repos
}

fn append_modeldir_args_if_missing(args: &mut Vec<String>, workdir: &Path, repositories: &[String]) {
    if has_modeldir_arg(args) {
        return;
    }

    let mut entries: Vec<String> = vec![workdir.to_string_lossy().to_string()];
    for repo in repositories {
        let value = repo.trim();
        if value.is_empty() {
            continue;
        }
        if !entries.iter().any(|e| e == value) {
            entries.push(value.to_string());
        }
    }

    args.push("--modeldir".to_string());
    args.push(entries.join(";"));
}

fn has_modeldir_arg(args: &[String]) -> bool {
    args.iter().any(|arg| arg == "--modeldir" || arg.starts_with("--modeldir="))
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
