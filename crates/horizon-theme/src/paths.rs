use std::path::PathBuf;

/// Resolves a path from an XDG-style environment variable.
///
/// An absolute value from `var` takes precedence. If the variable is unset or it
/// contains a relative path, the path is resolved relative to `HOME` using `fallback`.
///
/// Returns `None` when neither source provides a path.
pub fn xdg(var: &str, fallback: &str) -> Option<PathBuf> {
    let home = std::env::var_os("HOME").map(PathBuf::from);
    std::env::var_os(var)
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(|| home.as_ref().map(|h| h.join(fallback)))
}
