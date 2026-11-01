// Fiche « À propos de Casier »: Casier est le nom affiché de fili-file (alias), fili-file reste le nom du programme.
// Libellés traduits (i18n::tr), valeurs telles quelles.

pub const NAME: &str = "Casier";
pub const BINARY: &str = "fili-file";

pub fn version() -> String {
    match option_env!("FILI_FILE_GIT") {
        Some(g) if !g.is_empty() => format!("{} ({g})", env!("CARGO_PKG_VERSION")),
        _ => env!("CARGO_PKG_VERSION").to_string(),
    }
}

// Logo de Filigrane posé sur le dossier: blanc sur le dossier sombre (thème clair), ardoise sur le dossier clair
pub fn logo(light: bool) -> String {
    let home = std::env::var("HOME").unwrap_or_default();
    format!("file://{home}/.local/share/filigrane/filigrane-mono-{}.svg", if light { "blanc" } else { "ardoise" })
}

pub fn rows() -> Vec<(&'static str, String)> {
    vec![
        (crate::i18n::tr("Version"), version()),
        (crate::i18n::tr("Also known as"), BINARY.to_string()),
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
        assert!(super::rows().iter().any(|(k, v)| v == "fili-file"));
    }
}
