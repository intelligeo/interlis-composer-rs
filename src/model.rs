use std::collections::HashMap;

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
    pub enums: Vec<EnumDef>,
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
pub struct EnumDef {
    pub name: String,
    #[serde(default)]
    pub values: Vec<String>,
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
                enums: Vec::new(),
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
    matches!(
        normalize_type(value).as_str(),
        "TEXT" | "BOOLEAN" | "NUMERIC" | "DATE" | "TIMEOFDAY" | "DATETIME" | "UUID"
    )
}

fn normalize_type(value: &str) -> String {
    value.trim().to_ascii_uppercase()
}

fn to_interlis_type(value: &str) -> &'static str {
    match normalize_type(value).as_str() {
        "TEXT" => "TEXT*255",
        "BOOLEAN" => "BOOLEAN",
        "NUMERIC" => "NUMERIC",
        "DATE" => "INTERLIS.XMLDate",
        "TIMEOFDAY" => "INTERLIS.XMLTime",
        "DATETIME" => "INTERLIS.XMLDateTime",
        "UUID" => "INTERLIS.UUIDOID",
        _ => "TEXT*255",
    }
}

fn from_interlis_type(value: &str) -> String {
    let normalized = value.trim().to_ascii_uppercase();
    if normalized.starts_with("TEXT") {
        return "TEXT".to_string();
    }
    match normalized.as_str() {
        "BOOLEAN" => "BOOLEAN".to_string(),
        "NUMERIC" => "NUMERIC".to_string(),
        "INTERLIS.XMLDATE" => "DATE".to_string(),
        "INTERLIS.XMLTIME" => "TIMEOFDAY".to_string(),
        "INTERLIS.XMLDATETIME" => "DATETIME".to_string(),
        "INTERLIS.UUIDOID" => "UUID".to_string(),
        _ => "TEXT".to_string(),
    }
}

pub fn from_interlis(input: &str) -> Result<InterlisModel, ParseFailure> {
    let mut model_name: Option<String> = None;
    let mut language: Option<String> = None;
    let mut model_version: Option<String> = None;
    let mut classes: Vec<Class> = Vec::new();
    let mut current_class: Option<Class> = None;
    let mut first_code_line_seen = false;

    for (idx, raw_line) in input.lines().enumerate() {
        let line_no = idx + 1;
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with("!!") {
            continue;
        }

        if !first_code_line_seen {
            first_code_line_seen = true;
            if line.starts_with("INTERLIS ") {
                model_version = Some(line.trim_end_matches(';').trim().to_string());
            } else if line.eq_ignore_ascii_case("TRANSFER INTERLIS1;") {
                model_version = Some("TRANSFER INTERLIS1".to_string());
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

        if line.starts_with("CLASS ") && line.ends_with('=') {
            if current_class.is_some() {
                return Err(ParseFailure::new(
                    "Nested CLASS definitions are not supported",
                    line_no,
                    1,
                ));
            }
            let class_name = line
                .strip_prefix("CLASS ")
                .and_then(|v| v.strip_suffix('='))
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty())
                .ok_or_else(|| ParseFailure::new("Invalid CLASS declaration", line_no, 1))?;

            current_class = Some(Class {
                name: class_name,
                attributes: Vec::new(),
            });
            continue;
        }

        if line.starts_with("END ") {
            if let Some(class) = current_class.take() {
                classes.push(class);
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

    let parsed_model_name =
        model_name.ok_or_else(|| ParseFailure::new("MODEL declaration not found", 1, 1))?;

    let md5 = format!("{:x}", md5::compute(input.as_bytes()));

    Ok(InterlisModel {
        model_name: parsed_model_name,
        language: language.unwrap_or_else(default_language),
        model_version: model_version.unwrap_or_else(|| "INTERLIS 2.4".to_string()),
        model_md5: md5,
        domains: Vec::new(),
        associations: Vec::new(),
        enums: Vec::new(),
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
        out.push_str(&format!("MODEL {} ({}) =\n", self.model_name, self.language));
        out.push_str("  TOPIC Main =\n");

        for topic in &self.topics {
            out.push_str(&format!("    !! TOPIC {}\n", topic.name));
            for subtopic in &topic.subtopics {
                out.push_str(&format!("    !! SUBTOPIC {}.{}\n", topic.name, subtopic.name));
            }
        }

        for domain in &self.domains {
            out.push_str(&format!(
                "    !! DOMAIN {}: {};\n",
                domain.name,
                to_interlis_type(&domain.r#type)
            ));
        }

        for en in &self.enums {
            let values = if en.values.is_empty() {
                "".to_string()
            } else {
                en.values.join(", ")
            };
            out.push_str(&format!("    !! ENUM {} = ({});\n", en.name, values));
        }

        for class in &self.classes {
            out.push_str(&format!("    CLASS {} =\n", class.name));
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
                "    !! ASSOCIATION {} ({} -> {}, {});\n",
                assoc.name, assoc.from_class, assoc.to_class, assoc.cardinality
            ));
        }

        out.push_str("  END Main;\n");
        out.push_str("END ");
        out.push_str(&self.model_name);
        out.push_str(".\n");
        out
    }
}
