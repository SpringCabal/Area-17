use std::{env, fs, path::PathBuf};

fn main() {
    println!("cargo:rustc-link-arg=--initial-memory=67108864");
    println!("cargo:rustc-link-arg=--no-growable-memory");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=CARGO_CFG_TARGET_ARCH");
    generate_scenario_data();
    let _ = env::var_os("CARGO_CFG_TARGET_ARCH");
}

fn generate_scenario_data() {
    let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let source_path = manifest_dir.join("../../LuaRules/Configs/spawn.lua");
    println!("cargo:rerun-if-changed={}", source_path.display());
    let source = fs::read_to_string(&source_path).unwrap_or_else(|error| {
        panic!("failed to read Area 17 scenario source {source_path:?}: {error}")
    });

    let mut units = Vec::new();
    let mut current = ScenarioUnit::default();
    let mut in_units = false;
    for line in source.lines() {
        if line.trim() == "units = {" {
            in_units = true;
            continue;
        }
        if !in_units {
            continue;
        }
        if line.trim_start().starts_with('[') && line.contains("] = {") {
            if current.is_complete() {
                units.push(current);
            }
            current = ScenarioUnit::default();
            continue;
        }
        let trimmed = line.trim();
        if let Some(value) = trimmed.strip_prefix("unitDefName = ") {
            current.name = Some(parse_string(value));
        } else if let Some(value) = trimmed.strip_prefix("tooltip = ") {
            current.tooltip = Some(parse_string(value));
        } else if let Some(value) = trimmed.strip_prefix("teamId = ") {
            current.team = Some(parse_number(value));
        } else if let Some(value) = trimmed.strip_prefix("angle = ") {
            current.angle = Some(parse_number(value));
        } else if let Some(value) = trimmed.strip_prefix("x = ") {
            current.x = Some(parse_number(value));
        } else if let Some(value) = trimmed.strip_prefix("y = ") {
            current.y = Some(parse_number(value));
        } else if let Some(value) = trimmed.strip_prefix("z = ") {
            current.z = Some(parse_number(value));
        } else if let Some(value) = trimmed.strip_prefix("neutral = ") {
            current.neutral = Some(parse_bool(value));
        } else if let Some(value) = trimmed.strip_prefix("experience = ") {
            current.experience = Some(parse_number(value));
        } else if let Some(value) = trimmed.strip_prefix("n = ") {
            current.rules_n = Some(parse_number(value));
        } else if let Some(value) = trimmed.strip_prefix("firestate = ") {
            current.firestate = Some(parse_number(value));
        } else if let Some(value) = trimmed.strip_prefix("movestate = ") {
            current.movestate = Some(parse_number(value));
        } else if let Some(value) = trimmed.strip_prefix("cloak = ") {
            current.cloak = Some(parse_bool(value));
        } else if let Some(value) = trimmed.strip_prefix("[\"repeat\"] = ") {
            current.repeat = Some(parse_bool(value));
        }
    }
    if current.is_complete() {
        units.push(current);
    }
    assert_eq!(units.len(), 349, "unexpected Area 17 scenario unit count");

    let output_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let output_path = output_dir.join("scenario_generated.rs");
    let mut output = String::from("pub static INITIAL_UNITS: &[InitialUnit] = &[\n");
    for unit in units {
        let name = unit.name.as_ref().unwrap();
        output.push_str(&format!(
            "    InitialUnit {{ name: {:?}, tooltip: {:?}, x: {}f32, y: {}f32, z: {}f32, angle: {}f32, team: {}, neutral: {}, experience: {}f32, rules_n: {}f32, firestate: {}, movestate: {}, cloak: {}, repeat: {} }},\n",
            name,
            unit.tooltip.unwrap_or_default(),
            float_literal(unit.x.unwrap()),
            float_literal(unit.y.unwrap()),
            float_literal(unit.z.unwrap()),
            float_literal(unit.angle.unwrap_or(0.0)),
            unit.team.unwrap_or(0),
            unit.neutral.unwrap_or(false),
            float_literal(unit.experience.unwrap_or(0.0)),
            float_literal(unit.rules_n.unwrap_or(0.0)),
            unit.firestate.unwrap_or_else(|| {
                if unit.name.as_deref() == Some("ufo") {
                    0
                } else {
                    2
                }
            }),
            unit.movestate.unwrap_or(0),
            unit.cloak.unwrap_or(false),
            unit.repeat.unwrap_or(false),
        ));
    }
    output.push_str("];\n");
    fs::write(output_path, output).unwrap();
}

#[derive(Default)]
struct ScenarioUnit {
    name: Option<String>,
    tooltip: Option<String>,
    x: Option<f32>,
    y: Option<f32>,
    z: Option<f32>,
    angle: Option<f32>,
    team: Option<i32>,
    neutral: Option<bool>,
    experience: Option<f32>,
    rules_n: Option<f32>,
    firestate: Option<i32>,
    movestate: Option<i32>,
    cloak: Option<bool>,
    repeat: Option<bool>,
}

impl ScenarioUnit {
    fn is_complete(&self) -> bool {
        self.name.is_some() && self.x.is_some() && self.y.is_some() && self.z.is_some()
    }
}

fn parse_string(value: &str) -> String {
    value
        .trim()
        .trim_end_matches(',')
        .trim_matches('"')
        .to_owned()
}

fn float_literal(value: f32) -> String {
    format!("{value:?}")
}

fn parse_number<T: std::str::FromStr>(value: &str) -> T
where
    T::Err: std::fmt::Debug,
{
    value.trim().trim_end_matches(',').parse().unwrap()
}

fn parse_bool(value: &str) -> bool {
    value.trim().trim_end_matches(',') == "true"
}
