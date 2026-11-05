#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorSchemePreference { Default, ForceDark, ForceLight }

pub struct ThemeSupport;

impl ThemeSupport {
    pub fn resolve_mode(pref: ColorSchemePreference, system_dark: bool) -> bool {
        match pref {
            ColorSchemePreference::ForceDark => true,
            ColorSchemePreference::ForceLight => false,
            ColorSchemePreference::Default => system_dark,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_theme_support_resolve_aaa() {
        // Arrange & Act
        let d1 = ThemeSupport::resolve_mode(ColorSchemePreference::ForceDark, false);
        let d2 = ThemeSupport::resolve_mode(ColorSchemePreference::Default, true);
        // Assert
        assert!(d1);
        assert!(d2);
    }
}
