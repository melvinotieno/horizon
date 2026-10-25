use std::{collections::HashMap, path::Path};

use crate::{color::Rgba, paths::xdg};

const BUILTIN: &str = include_str!("../assets/tokens.toml");
const TOKEN_RELATIVE_PATH: &str = "horizon/tokens.toml";

/// A partial tokens file to merge on top of the built-in tokens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Layer {
    pub label: String,
    pub source: String,
}

/// Every color of a particular color scheme, plus its elevation adjustments.
#[derive(Debug, Clone, PartialEq)]
pub struct Colors {
    colors: HashMap<String, Rgba>,
}

impl Colors {
    /// Returns the color for a given role, or a fallback if the role is not found.
    #[track_caller]
    pub fn color(&self, role: &str) -> Rgba {
        lookup(&self.colors, "color", role, Rgba::new(1.0, 0.0, 1.0, 1.0))
    }

    /// Returns true if the color role exists in this color scheme.
    pub fn has(&self, role: &str) -> bool {
        self.colors.contains_key(role)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FontStyle {
    pub size: f32,
    pub line: f32,
    pub weight: f32,
    pub tracking: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Tokens {
    pub light: Colors,
    pub dark: Colors,
    pub font_family: String,
    pub font_family_mono: String,
    pub font_features: String,
    font_styles: HashMap<String, FontStyle>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error(pub String);

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for Error {}

impl Tokens {
    /// Loads and compiles the built-in tokens with optional system and user layers.
    ///
    /// The layers are loaded in the following order:
    /// 1. Built-in tokens (always present)
    /// 2. `/usr/share/horizon/tokens.toml` (optional)
    /// 3. `$XDG_CONFIG_HOME/horizon/tokens.toml` or `$HOME/.config/horizon/tokens.toml` (optional)
    ///
    /// Each layer overrides matching values from earlier layers.
    ///
    /// If a layer is not found or cannot be read, it will be ignored with a warning.
    pub fn load() -> Result<Self, Error> {
        let mut warnings = Vec::new();

        let system = read_layer(
            &Path::new("/usr/share").join(TOKEN_RELATIVE_PATH),
            &mut warnings,
        );

        let user = xdg("XDG_CONFIG_HOME", ".config")
            .map(|d| d.join(TOKEN_RELATIVE_PATH))
            .as_deref()
            .and_then(|p| read_layer(p, &mut warnings));

        let layers: Vec<Layer> = [system, user].into_iter().flatten().collect();
        let (tokens, more_warnings) = Self::load_with(&layers)?;

        for warning in warnings.into_iter().chain(more_warnings) {
            eprintln!("horizon: {warning}");
        }

        Ok(tokens)
    }

    /// Returns the font style for a given name, or a fallback if the name is not found.
    #[track_caller]
    pub fn font_style(&self, name: &str) -> FontStyle {
        let fallback = FontStyle {
            size: 14.0,
            line: 20.0,
            weight: 400.0,
            tracking: 0.0,
        };
        lookup(&self.font_styles, "font style", name, fallback)
    }

    fn load_with(layers: &[Layer]) -> Result<(Self, Vec<String>), Error> {
        let base: toml::Table = BUILTIN.parse().map_err(|e| Error(format!("{e}")))?;

        let mut tokens = Self::from_table(&base)?;
        let mut merged = base.clone();
        let mut warnings = Vec::new();

        for layer in layers {
            let mut overlay: toml::Table = match layer.source.parse() {
                Ok(t) => t,
                Err(e) => {
                    warnings.push(format!("{}: {e}; ignoring it", layer.label));
                    continue;
                }
            };

            overlay.remove("meta");

            let mut candidate = merged.clone();

            merge(&mut candidate, overlay);

            match Self::from_table(&candidate) {
                Ok(compiled) => {
                    merged = candidate;
                    tokens = compiled;
                }
                Err(e) => {
                    warnings.push(format!("{}: {e}; ignoring it", layer.label));
                }
            }
        }

        Ok((tokens, warnings))
    }

    /// Loads a token set from a TOML table, returning an error if the table is invalid.
    fn from_table(root: &toml::Table) -> Result<Self, Error> {
        // Traverses a TOML table using a path of keys.
        let table = |path: &[&str]| -> Result<&toml::Table, Error> {
            let mut current = root;
            for key in path {
                current = current
                    .get(*key)
                    .and_then(|v| v.as_table())
                    .ok_or_else(|| Error(format!("missing [{}]", path.join("."))))?;
            }
            Ok(current)
        };

        let palette = table(&["palette"])?;

        // Resolves a color reference to an Rgba value.
        let color = |reference: &str| -> Result<Rgba, Error> {
            if reference.starts_with('#') {
                return Rgba::from_hex(reference)
                    .ok_or_else(|| Error(format!("bad hex {reference:?}")));
            }
            let (ramp, step) = reference
                .split_once('.')
                .ok_or_else(|| Error(format!("bad color reference {reference:?}")))?;
            let hex = palette
                .get(ramp)
                .and_then(|r| r.get(step))
                .and_then(|v| v.as_str())
                .ok_or_else(|| Error(format!("no palette entry {reference:?}")))?;
            Rgba::from_hex(hex).ok_or_else(|| Error(format!("bad hex {hex:?}")))
        };

        // Loads a color scheme from the TOML table.
        let scheme = |name: &str| -> Result<Colors, Error> {
            let mut colors = HashMap::new();
            for (role, reference) in table(&["color", name])? {
                let reference = reference
                    .as_str()
                    .ok_or_else(|| Error(format!("color.{name}.{role} must be a string")))?;
                colors.insert(role.clone(), color(reference)?);
            }
            Ok(Colors { colors })
        };

        // Loads the light and dark color schemes.
        let (light, dark) = (scheme("light")?, scheme("dark")?);
        for role in light.colors.keys() {
            if !dark.has(role) {
                return Err(Error(format!(
                    "color role {role} is missing from the dark color scheme"
                )));
            }
        }

        let typography = table(&["typography"])?;

        // Resolves font details from the TOML table.
        let font = |key: &str, default: &str| -> String {
            typography
                .get(key)
                .and_then(|v| v.as_str())
                .unwrap_or(default)
                .to_string()
        };

        // Resolves font features from the TOML table.
        let font_features = typography
            .get("features")
            .and_then(|v| v.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|f| f.as_str())
                    .map(|f| format!("\"{f}\" 1"))
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_default();

        // Loads font styles from the TOML table.
        let mut font_styles = HashMap::new();
        for (name, value) in table(&["typography", "style"])? {
            let style = |k: &str| value.get(k).and_then(num).unwrap_or(0.0) as f32;
            font_styles.insert(
                name.clone(),
                FontStyle {
                    size: style("size"),
                    line: style("line"),
                    weight: style("weight"),
                    tracking: style("tracking"),
                },
            );
        }

        Ok(Self {
            light,
            dark,
            font_family: font("family", "Inter"),
            font_family_mono: font("family_mono", "monospace"),
            font_features,
            font_styles,
        })
    }
}

/// Looks up a value in a map by name, returning a fallback if the name is not found.
#[track_caller]
fn lookup<T: Copy>(map: &HashMap<String, T>, kind: &str, name: &str, fallback: T) -> T {
    match map.get(name) {
        Some(value) => *value,
        None => {
            // In debug builds, panic if the name is not found. This helps catch typos
            // and other mistakes during development. In release builds, return the fallback
            debug_assert!(false, "unknown {kind} {name:?} (not in tokens.toml)");
            fallback
        }
    }
}

/// Reads a layer from a file, returning None if the file is not found.
fn read_layer(path: &Path, warnings: &mut Vec<String>) -> Option<Layer> {
    match std::fs::read_to_string(path) {
        Ok(source) => Some(Layer {
            label: path.display().to_string(),
            source,
        }),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => {
            warnings.push(format!("{}: {e}; ignoring it", path.display()));
            None
        }
    }
}

/// Merges the overlay table into the base table, recursively merging nested tables.
fn merge(base: &mut toml::Table, overlay: toml::Table) {
    for (key, value) in overlay {
        match (base.get_mut(&key), value) {
            (Some(toml::Value::Table(b)), toml::Value::Table(o)) => merge(b, o),
            (_, value) => {
                base.insert(key, value);
            }
        }
    }
}

/// Converts a TOML value to a number, if possible.
fn num(v: &toml::Value) -> Option<f64> {
    v.as_float().or_else(|| v.as_integer().map(|i| i as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge() {
        let mut base: toml::Table = r#"
            [a]
            b = 1
            [a.c]
            d = 2
        "#
        .parse()
        .unwrap();
        let overlay: toml::Table = r#"
            [a]
            b = 3
            [a.e]
            f = 4
        "#
        .parse()
        .unwrap();
        merge(&mut base, overlay);
        assert_eq!(
            base.get("a")
                .unwrap()
                .get("b")
                .unwrap()
                .as_integer()
                .unwrap(),
            3
        );
        assert_eq!(
            base.get("a")
                .unwrap()
                .get("c")
                .unwrap()
                .get("d")
                .unwrap()
                .as_integer()
                .unwrap(),
            2
        );
        assert_eq!(
            base.get("a")
                .unwrap()
                .get("e")
                .unwrap()
                .get("f")
                .unwrap()
                .as_integer()
                .unwrap(),
            4
        );
    }
}
