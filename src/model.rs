use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InterlisModel {
    pub model_name: String,
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default)]
    pub model_version: String,
    #[serde(default)]
    pub model_md5: String,
    #[serde(default)]
    pub domains: Vec<Domain>,
    #[serde(default)]
    pub associations: Vec<Association>,
    #[serde(default)]
    pub topics: Vec<Topic>,
    #[serde(default)]
    pub classes: Vec<Class>,
    #[serde(default)]
    pub prose_sections: Vec<ProseSection>,
    #[serde(default)]
    pub catalog_instances: Vec<CatalogInstance>,
    #[serde(default)]
    pub catalog_validation_presets: Vec<CatalogValidationPreset>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Domain {
    pub name: String,
    #[serde(rename = "type")]
    pub r#type: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Association {
    pub name: String,
    #[serde(default)]
    pub from_class: String,
    #[serde(default)]
    pub to_class: String,
    #[serde(default = "default_cardinality")]
    pub cardinality: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Topic {
    pub name: String,
    #[serde(default)]
    pub subtopics: Vec<SubTopic>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SubTopic {
    pub name: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Class {
    pub name: String,
    #[serde(default, rename = "abstract")]
    pub is_abstract: bool,
    #[serde(default)]
    pub extends: String,
    #[serde(default)]
    pub attributes: Vec<Attribute>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Attribute {
    pub name: String,
    #[serde(rename = "type")]
    pub r#type: String,
    #[serde(default)]
    pub mandatory: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProseSection {
    pub title: String,
    #[serde(default)]
    pub content: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CatalogInstance {
    #[serde(default)]
    pub catalog: String,
    #[serde(default)]
    pub code: String,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub class_ref: String,
    #[serde(default)]
    pub properties: HashMap<String, String>,
    #[serde(default)]
    pub validation_preset: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CatalogValidationRule {
    #[serde(default)]
    pub attribute: String,
    #[serde(default)]
    pub domain_ref: String,
    #[serde(default)]
    pub expected_type: String,
    #[serde(default)]
    pub mandatory: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CatalogValidationPreset {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub target_class: String,
    #[serde(default = "default_preset_version")]
    pub version: u16,
    #[serde(default)]
    pub owner: String,
    #[serde(default = "default_preset_review_status")]
    pub review_status: String,
    #[serde(default)]
    pub approved_by: String,
    #[serde(default)]
    pub approved_at: String,
    #[serde(default)]
    pub rules: Vec<CatalogValidationRule>,
}

fn default_language() -> String {
    "de".to_string()
}

fn canonical_interlis_version(raw: &str) -> Option<&'static str> {
    let value = raw.trim().trim_end_matches(';').trim();
    if value.eq_ignore_ascii_case("INTERLIS 2.3") {
        Some("INTERLIS 2.3")
    } else if value.eq_ignore_ascii_case("INTERLIS 2.4") {
        Some("INTERLIS 2.4")
    } else {
        None
    }
}

fn default_project_version() -> String {
    "0.1".to_string()
}

fn default_diagram_zoom() -> u16 {
    100
}

fn default_diagram_hide_attributes() -> bool {
    false
}

fn default_diagram_anchor_snap() -> bool {
    false
}

fn default_diagram_layout_locked() -> bool {
    false
}

fn default_diagram_lock_nodes() -> bool {
    false
}

fn default_diagram_lock_relations() -> bool {
    false
}

fn default_diagram_preset() -> String {
    "default".to_string()
}

fn default_diagram_preset_context_filter() -> String {
    "all".to_string()
}

fn default_preset_context() -> String {
    "authoring".to_string()
}

fn default_preset_version() -> u16 {
    1
}

fn default_preset_review_status() -> String {
    "draft".to_string()
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DiagramNodePosition {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DiagramPresetConfig {
    #[serde(default)]
    pub hide_attributes: bool,
    #[serde(default)]
    pub anchor_snap: bool,
    #[serde(default)]
    pub lock_nodes: bool,
    #[serde(default)]
    pub lock_relations: bool,
    #[serde(default = "default_preset_context")]
    pub context: String,
    #[serde(default = "default_preset_version")]
    pub version: u16,
    #[serde(default)]
    pub owner: String,
    #[serde(default = "default_preset_review_status")]
    pub review_status: String,
    #[serde(default)]
    pub approved_by: String,
    #[serde(default)]
    pub approved_at: String,
}

fn default_cardinality() -> String {
    "1..*".to_string()
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UiState {
    #[serde(default)]
    pub open_editors: Vec<String>,
    #[serde(default)]
    pub active_editor: String,
    #[serde(default)]
    pub layout: String,
    #[serde(default = "default_diagram_zoom")]
    pub diagram_zoom: u16,
    #[serde(default)]
    pub diagram_positions: HashMap<String, DiagramNodePosition>,
    #[serde(default = "default_diagram_hide_attributes")]
    pub diagram_hide_attributes: bool,
    #[serde(default)]
    pub diagram_relation_offsets: HashMap<String, f32>,
    #[serde(default = "default_diagram_anchor_snap")]
    pub diagram_anchor_snap: bool,
    #[serde(default = "default_diagram_layout_locked")]
    pub diagram_layout_locked: bool,
    #[serde(default = "default_diagram_lock_nodes")]
    pub diagram_lock_nodes: bool,
    #[serde(default = "default_diagram_lock_relations")]
    pub diagram_lock_relations: bool,
    #[serde(default = "default_diagram_preset")]
    pub diagram_preset: String,
    #[serde(default = "default_diagram_preset_context_filter")]
    pub diagram_preset_context_filter: String,
    #[serde(default)]
    pub diagram_custom_presets: HashMap<String, DiagramPresetConfig>,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            open_editors: vec!["text".to_string()],
            active_editor: "text".to_string(),
            layout: "single".to_string(),
            diagram_zoom: default_diagram_zoom(),
            diagram_positions: HashMap::new(),
            diagram_hide_attributes: default_diagram_hide_attributes(),
            diagram_relation_offsets: HashMap::new(),
            diagram_anchor_snap: default_diagram_anchor_snap(),
            diagram_layout_locked: default_diagram_layout_locked(),
            diagram_lock_nodes: default_diagram_lock_nodes(),
            diagram_lock_relations: default_diagram_lock_relations(),
            diagram_preset: default_diagram_preset(),
            diagram_preset_context_filter: default_diagram_preset_context_filter(),
            diagram_custom_presets: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InterlisProject {
    pub project_name: String,
    #[serde(default = "default_project_version")]
    pub project_version: String,
    #[serde(default)]
    pub ui_state: UiState,
    pub model: InterlisModel,
}

impl InterlisProject {
    pub fn empty(project_name: String, model_name: String, language: String) -> Self {
        Self {
            project_name,
            project_version: default_project_version(),
            ui_state: UiState::default(),
            model: InterlisModel {
                model_name,
                language,
                model_version: String::new(),
                model_md5: String::new(),
                domains: Vec::new(),
                associations: Vec::new(),
                topics: Vec::new(),
                classes: Vec::new(),
                prose_sections: Vec::new(),
                catalog_instances: Vec::new(),
                catalog_validation_presets: Vec::new(),
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct ParseFailure {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

impl ParseFailure {
    fn new(message: &str, line: usize, column: usize) -> Self {
        Self {
            message: message.to_string(),
            line,
            column,
        }
    }
}

pub fn is_supported_type(value: &str) -> bool {
    let normalized = normalize_type(value);
    if normalized.is_empty() {
        return false;
    }
    normalized == "TEXT"
        || normalized.starts_with("TEXT*")
        || normalized == "BOOLEAN"
        || normalized == "NUMERIC"
        || normalized.starts_with("NUMERIC(")
        || normalized == "DATE"
        || normalized == "TIMEOFDAY"
        || normalized == "DATETIME"
        || normalized == "UUID"
        || normalized == "INTERLIS.XMLDATE"
        || normalized == "INTERLIS.XMLTIME"
        || normalized == "INTERLIS.XMLDATETIME"
        || normalized == "INTERLIS.UUIDOID"
}

fn normalize_type(value: &str) -> String {
    value.trim().to_ascii_uppercase()
}

fn to_interlis_type(value: &str) -> String {
    let raw = value.trim();
    match normalize_type(raw).as_str() {
        "TEXT" => "TEXT*255".to_string(),
        "BOOLEAN" => "BOOLEAN".to_string(),
        "NUMERIC" => "NUMERIC".to_string(),
        "DATE" => "INTERLIS.XMLDate".to_string(),
        "TIMEOFDAY" => "INTERLIS.XMLTime".to_string(),
        "DATETIME" => "INTERLIS.XMLDateTime".to_string(),
        "UUID" => "INTERLIS.UUIDOID".to_string(),
        // Keep imported/raw INTERLIS types exactly as provided.
        _ => {
            if raw.is_empty() {
                "TEXT*255".to_string()
            } else {
                raw.to_string()
            }
        }
    }
}

fn from_interlis_type(value: &str) -> String {
    let raw = value.trim();
    if raw.is_empty() {
        return "TEXT".to_string();
    }
    raw.to_string()
}

fn parse_domain_declaration_line(line: &str) -> Option<(String, String)> {
    let body = line.trim().trim_end_matches(';').trim();
    let (name_part, type_part) = body.split_once('=')?;
    let domain_name = name_part.trim();
    if domain_name.is_empty() {
        return None;
    }
    Some((domain_name.to_string(), from_interlis_type(type_part.trim())))
}

fn parse_association_end_line(line: &str, class_names: &HashSet<String>) -> Option<(String, String)> {
    let body = line.trim().trim_end_matches(';').trim();
    let open_idx = body.find('{')?;
    let close_rel_idx = body[open_idx + 1..].find('}')?;
    let close_idx = open_idx + 1 + close_rel_idx;
    let cardinality = body[open_idx + 1..close_idx].trim().to_string();

    // Try class token immediately after cardinality.
    let after_token = body[close_idx + 1..]
        .split_whitespace()
        .next()
        .unwrap_or("")
        .trim();
    if !after_token.is_empty() && class_names.contains(after_token) {
        return Some((after_token.to_string(), cardinality));
    }

    // Fallback: find any known class token before the connector/cardinality.
    let before_part = &body[..open_idx];
    for token in before_part
        .split(|c: char| c.is_whitespace() || c == '-' || c == '<' || c == '>' || c == '(' || c == ')' || c == ',')
        .filter(|tok| !tok.is_empty())
    {
        if class_names.contains(token) {
            return Some((token.to_string(), cardinality));
        }
    }

    // Last fallback keeps historical behavior.
    if !after_token.is_empty() {
        return Some((after_token.to_string(), cardinality));
    }

    None
}

pub fn from_interlis(input: &str) -> Result<InterlisModel, ParseFailure> {
    let mut model_name: Option<String> = None;
    let mut language: Option<String> = None;
    let mut model_version: Option<String> = None;
    let mut domains: Vec<Domain> = Vec::new();
    let mut associations: Vec<Association> = Vec::new();
    let mut classes: Vec<Class> = Vec::new();
    let mut current_class: Option<Class> = None;
    let mut current_association_name: Option<String> = None;
    let mut current_association_ends: Vec<(String, String)> = Vec::new();
    let mut in_domain_block = false;
    let mut first_code_line_seen = false;

    for (idx, raw_line) in input.lines().enumerate() {
        let line_no = idx + 1;
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }

        if line.starts_with("!! ASSOCIATION ") {
            // Backward compatibility with historical pseudo-export format:
            // !! ASSOCIATION Name (From -> To, 1..*);
            if let Some(body) = line.strip_prefix("!! ASSOCIATION ") {
                let body = body.trim().trim_end_matches(';').trim();
                if let Some((name_part, rest)) = body.split_once('(') {
                    let assoc_name = name_part.trim();
                    let rest = rest.trim_end_matches(')').trim();
                    if let Some((pair_part, card_part)) = rest.rsplit_once(',') {
                        let card = card_part.trim();
                        if let Some((from_part, to_part)) = pair_part.split_once("->") {
                            let from_class = from_part.trim();
                            let to_class = to_part.trim();
                            if !assoc_name.is_empty() && !from_class.is_empty() && !to_class.is_empty() {
                                associations.push(Association {
                                    name: assoc_name.to_string(),
                                    from_class: from_class.to_string(),
                                    to_class: to_class.to_string(),
                                    cardinality: card.to_string(),
                                });
                            }
                        }
                    }
                }
            }
            continue;
        }

        if line.starts_with("!! DOMAIN ") {
            // Backward compatibility with historical pseudo-export format:
            // !! DOMAIN Name: TYPE;
            if let Some(body) = line.strip_prefix("!! DOMAIN ") {
                let body = body.trim().trim_end_matches(';').trim();
                if let Some((name_part, type_part)) = body.split_once(':') {
                    let domain_name = name_part.trim();
                    if !domain_name.is_empty() {
                        domains.push(Domain {
                            name: domain_name.to_string(),
                            r#type: from_interlis_type(type_part.trim()),
                        });
                    }
                }
            }
            continue;
        }

        if line.starts_with("!!") {
            continue;
        }

        if !first_code_line_seen {
            first_code_line_seen = true;
            if line.starts_with("INTERLIS ") {
                let version = line.trim_end_matches(';').trim();
                if let Some(canonical) = canonical_interlis_version(version) {
                    model_version = Some(canonical.to_string());
                } else {
                    return Err(ParseFailure::new(
                        "Unsupported INTERLIS version: supported versions are INTERLIS 2.3 and INTERLIS 2.4",
                        line_no,
                        1,
                    ));
                }
            } else if line.eq_ignore_ascii_case("TRANSFER INTERLIS1;") {
                return Err(ParseFailure::new(
                    "Unsupported INTERLIS version: INTERLIS 1 is not supported (requires INTERLIS 2.3 or 2.4)",
                    line_no,
                    1,
                ));
            }
        }

        if line.starts_with("MODEL ") {
            let body = line.strip_prefix("MODEL ").ok_or_else(|| {
                ParseFailure::new("Invalid MODEL declaration", line_no, 1)
            })?;
            let (name_part, rest) = body
                .split_once('(')
                .ok_or_else(|| {
                    ParseFailure::new(
                        "MODEL declaration must include language in parentheses",
                        line_no,
                        1,
                    )
                })?;
            let (lang_part, _) = rest
                .split_once(')')
                .ok_or_else(|| {
                    ParseFailure::new(
                        "MODEL declaration has unclosed language parentheses",
                        line_no,
                        1,
                    )
                })?;

            let parsed_name = name_part.trim().to_string();
            if parsed_name.is_empty() {
                return Err(ParseFailure::new("MODEL name cannot be empty", line_no, 1));
            }

            model_name = Some(parsed_name);
            language = Some(lang_part.trim().to_string());
            continue;
        }

        if in_domain_block {
            if line.eq_ignore_ascii_case("CLASS")
                || line.starts_with("CLASS ")
                || line.starts_with("ABSTRACT CLASS ")
                || line.starts_with("ASSOCIATION ")
                || line.starts_with("END ")
            {
                in_domain_block = false;
            } else {
                if line.ends_with(';') {
                    if let Some((domain_name, domain_type)) = parse_domain_declaration_line(line) {
                        domains.push(Domain {
                            name: domain_name,
                            r#type: domain_type,
                        });
                    }
                }
                continue;
            }
        }

        if line.eq_ignore_ascii_case("DOMAIN") {
            in_domain_block = true;
            continue;
        }

        if line.starts_with("DOMAIN ") && line.ends_with(';') {
            let domain_line = line.strip_prefix("DOMAIN ").unwrap_or("").trim();
            if let Some((domain_name, domain_type)) = parse_domain_declaration_line(domain_line) {
                domains.push(Domain {
                    name: domain_name,
                    r#type: domain_type,
                });
            }
            continue;
        }

        if line.ends_with('=') && line.starts_with("ASSOCIATION ") {
            if current_association_name.is_some() {
                return Err(ParseFailure::new(
                    "Nested ASSOCIATION definitions are not supported",
                    line_no,
                    1,
                ));
            }

            let assoc_name = line
                .trim_end_matches('=')
                .trim()
                .strip_prefix("ASSOCIATION ")
                .map(str::trim)
                .unwrap_or("");
            if assoc_name.is_empty() {
                return Err(ParseFailure::new("ASSOCIATION name cannot be empty", line_no, 1));
            }

            current_association_name = Some(assoc_name.to_string());
            current_association_ends.clear();
            continue;
        }

        if line.ends_with('=') {
            if current_class.is_some() {
                return Err(ParseFailure::new(
                    "Nested CLASS definitions are not supported",
                    line_no,
                    1,
                ));
            }

            let decl = line.trim_end_matches('=').trim();
            let mut tokens = decl.split_whitespace();
            let first = tokens.next();
            let mut is_abstract = false;
            let class_kw = match first {
                Some(tok) if tok.eq_ignore_ascii_case("ABSTRACT") => {
                    is_abstract = true;
                    tokens.next()
                }
                _ => first,
            };

            if let Some(class_token) = class_kw {
                if class_token.eq_ignore_ascii_case("CLASS") {
                    let name_raw = tokens.next().unwrap_or("").trim();
                    if name_raw.is_empty() {
                        return Err(ParseFailure::new("CLASS name cannot be empty", line_no, 1));
                    }

                    let mut extends_raw = String::new();
                    let mut rest = tokens.peekable();
                    while let Some(token) = rest.next() {
                        if token.eq_ignore_ascii_case("(ABSTRACT)")
                            || token.eq_ignore_ascii_case("ABSTRACT")
                        {
                            is_abstract = true;
                            continue;
                        }

                        if token.eq_ignore_ascii_case("EXTENDS")
                            || token.eq_ignore_ascii_case("EXTEND")
                            || token.eq_ignore_ascii_case("EXTENT")
                        {
                            let base_name = rest.next().unwrap_or("").trim();
                            if base_name.is_empty() {
                                return Err(ParseFailure::new(
                                    "Invalid EXTENDS clause in CLASS declaration",
                                    line_no,
                                    1,
                                ));
                            }
                            extends_raw = base_name.to_string();
                            continue;
                        }

                        return Err(ParseFailure::new(
                            "Invalid CLASS declaration",
                            line_no,
                            1,
                        ));
                    }

                    current_class = Some(Class {
                        name: name_raw.to_string(),
                        is_abstract,
                        extends: extends_raw,
                        attributes: Vec::new(),
                    });
                    continue;
                }
            }
        }

        if line.starts_with("END ") {
            if let Some(class) = current_class.take() {
                classes.push(class);
                continue;
            }

            if let Some(assoc_name) = current_association_name.take() {
                if current_association_ends.len() >= 2 {
                    let (from_class, from_cardinality) = current_association_ends[0].clone();
                    let (to_class, to_cardinality) = current_association_ends[1].clone();
                    associations.push(Association {
                        name: assoc_name,
                        from_class,
                        to_class,
                        cardinality: if !to_cardinality.is_empty() {
                            to_cardinality
                        } else {
                            from_cardinality
                        },
                    });
                }
                current_association_ends.clear();
            }
            continue;
        }

        if current_association_name.is_some() {
            if line.ends_with(';') {
                let class_names: HashSet<String> = classes.iter().map(|c| c.name.clone()).collect();
                if let Some((class_name, cardinality)) = parse_association_end_line(line, &class_names) {
                    current_association_ends.push((class_name, cardinality));
                }
            }
            continue;
        }

        if let Some(class) = current_class.as_mut() {
            if let Some((name_part, rhs_part)) = line.split_once(':') {
                let attr_name = name_part.trim().to_string();
                if attr_name.is_empty() {
                    return Err(ParseFailure::new(
                        "Attribute name cannot be empty",
                        line_no,
                        1,
                    ));
                }

                let rhs = rhs_part.trim().trim_end_matches(';').trim();
                let (mandatory, type_part) = if let Some(rest) = rhs.strip_prefix("MANDATORY ") {
                    (true, rest.trim())
                } else {
                    (false, rhs)
                };

                let attr_type = from_interlis_type(type_part);
                class.attributes.push(Attribute {
                    name: attr_name,
                    r#type: attr_type,
                    mandatory,
                });
            }
        }
    }

    if current_class.is_some() {
        return Err(ParseFailure::new("Unclosed CLASS block", input.lines().count(), 1));
    }

    if current_association_name.is_some() {
        return Err(ParseFailure::new(
            "Unclosed ASSOCIATION block",
            input.lines().count(),
            1,
        ));
    }

    let parsed_model_name =
        model_name.ok_or_else(|| ParseFailure::new("MODEL declaration not found", 1, 1))?;

    let md5 = format!("{:x}", md5::compute(input.as_bytes()));

    Ok(InterlisModel {
        model_name: parsed_model_name,
        language: language.unwrap_or_else(default_language),
        model_version: model_version.unwrap_or_else(|| "INTERLIS 2.4".to_string()),
        model_md5: md5,
        domains,
        associations,
        topics: Vec::new(),
        classes,
        prose_sections: Vec::new(),
        catalog_instances: Vec::new(),
        catalog_validation_presets: Vec::new(),
    })
}

impl InterlisModel {
    pub fn to_interlis(&self) -> String {
        let mut out = String::new();
        let version = if self.model_version.trim().is_empty() {
            "INTERLIS 2.4".to_string()
        } else {
            self.model_version.trim().to_string()
        };
        out.push_str(&format!("{};\n\n", version));
        out.push_str(&format!(
            "MODEL {} ({}) AT \"https://intelligeo.ch/imc\" =\n",
            self.model_name, self.language
        ));
        out.push_str("  TOPIC Main =\n");

        for topic in &self.topics {
            out.push_str(&format!("    !! TOPIC {}\n", topic.name));
            for subtopic in &topic.subtopics {
                out.push_str(&format!("    !! SUBTOPIC {}.{}\n", topic.name, subtopic.name));
            }
        }

        if !self.domains.is_empty() {
            out.push_str("    DOMAIN\n");
            for domain in &self.domains {
                out.push_str(&format!(
                    "      {} = {};\n",
                    domain.name,
                    to_interlis_type(&domain.r#type)
                ));
            }
            out.push('\n');
        }

        for class in &self.classes {
            let mut class_decl = String::new();
            if class.is_abstract {
                class_decl.push_str("ABSTRACT ");
            }
            class_decl.push_str("CLASS ");
            class_decl.push_str(&class.name);
            if !class.extends.trim().is_empty() {
                class_decl.push_str(" EXTENDS ");
                class_decl.push_str(class.extends.trim());
            }
            class_decl.push_str(" =");
            out.push_str("    ");
            out.push_str(&class_decl);
            out.push('\n');
            if class.attributes.is_empty() {
                out.push_str("      !! no attributes defined\n");
            } else {
                for attr in &class.attributes {
                    if attr.mandatory {
                        out.push_str(&format!(
                            "      {}: MANDATORY {};\n",
                            attr.name,
                            to_interlis_type(&attr.r#type)
                        ));
                    } else {
                        out.push_str(&format!(
                            "      {}: {};\n",
                            attr.name,
                            to_interlis_type(&attr.r#type)
                        ));
                    }
                }
            }
            out.push_str("    END ");
            out.push_str(&class.name);
            out.push_str(";\n\n");
        }

        for assoc in &self.associations {
            out.push_str(&format!(
                "    ASSOCIATION {} =\n",
                assoc.name
            ));
            out.push_str(&format!(
                "      from -- {{0..*}} {};\n",
                assoc.from_class
            ));
            out.push_str(&format!(
                "      to -- {{{}}} {};\n",
                assoc.cardinality,
                assoc.to_class
            ));
            out.push_str(&format!("    END {};\n\n", assoc.name));
        }

        out.push_str("  END Main;\n");
        out.push_str("END ");
        out.push_str(&self.model_name);
        out.push_str(".\n");
        out
    }
}
