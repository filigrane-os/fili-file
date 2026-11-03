// "About Casier" card: Casier is the display name of fili-file (an alias), fili-file stays the program name and
// `casier` works as a command too. Labels are translated (i18n::tr), values are shown as they are.

pub const BINARY: &str = "fili-file";
pub const ALIAS: &str = "casier";

// Crate version, plus the git description when it adds something (a development build between two tags); a release
// build made from the tag `v<version>` shows the version alone.
pub fn version() -> String {
    let v = env!("CARGO_PKG_VERSION");
    match option_env!("FILI_FILE_GIT") {
        Some(g) if !g.is_empty() && g != format!("v{v}") => format!("{v} ({g})"),
        _ => v.to_string(),
    }
}

// Filigrane logo laid on the folder: white on the dark folder (light theme), slate on the light folder
pub fn logo(light: bool) -> String {
    let home = std::env::var("HOME").unwrap_or_default();
    format!("file://{home}/.local/share/filigrane/filigrane-mono-{}.svg", if light { "blanc" } else { "ardoise" })
}

pub fn rows() -> Vec<(&'static str, String)> {
    vec![
        (crate::i18n::tr("Version"), version()),
        (crate::i18n::tr("Also known as"), BINARY.to_string()),
        (crate::i18n::tr("Commands"), format!("{ALIAS}, {BINARY}")),
        (crate::i18n::tr("Part of"), "Filigrane".to_string()),
        (crate::i18n::tr("Built with"), "Rust, Qt Quick".to_string()),
        (crate::i18n::tr("Settings"), crate::config::path().display().to_string().replace(&std::env::var("HOME").unwrap_or_default(), "~")),
    ]
}

#[cfg(test)]
mod tests {
    #[test]
    fn version_starts_with_the_crate_version() {
        assert!(super::version().starts_with(env!("CARGO_PKG_VERSION")));
        assert!(super::rows().iter().any(|(_, v)| v == "fili-file"));
    }

    #[test]
    fn both_commands_are_listed() {
        assert!(super::rows().iter().any(|(_, v)| v == "casier, fili-file"));
    }
}
