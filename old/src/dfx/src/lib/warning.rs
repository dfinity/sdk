pub enum DfxWarning {
    MainnetPlainTextIdentity,
}

pub fn is_warning_disabled(warning: DfxWarning) -> bool {
    let warning = match warning {
        DfxWarning::MainnetPlainTextIdentity => "mainnet_plaintext_identity",
    };
    // By default, warnings are all enabled.
    let env_warnings = std::env::var("DFX_WARNING").unwrap_or_else(|_| "".to_string());
    env_warnings
        .split(',')
        .filter(|w| w.starts_with('-'))
        .any(|w| w.chars().skip(1).collect::<String>().eq(warning))
}
