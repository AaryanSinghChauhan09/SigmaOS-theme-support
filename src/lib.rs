//! SigmaOS-theme-support — Theme support utilities for SigmaOS.

/// Color scheme preference
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ColorSchemePreference {
    /// Follow system
    #[default]
    Default,
    /// Force dark mode
    ForceDark,
    /// Force light mode
    ForceLight,
}

/// RGB color
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RgbColor {
    /// Create from hex string
    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 { return None; }
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        Some(Self { r, g, b })
    }

    /// To hex string
    pub fn to_hex(&self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }

    /// Relative luminance (WCAG)
    pub fn luminance(&self) -> f64 {
        fn channel(c: u8) -> f64 {
            let v = c as f64 / 255.0;
            if v <= 0.03928 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) }
        }
        0.2126 * channel(self.r) + 0.7152 * channel(self.g) + 0.0722 * channel(self.b)
    }

    /// Is dark color
    pub fn is_dark(&self) -> bool {
        self.luminance() < 0.5
    }

    /// Contrast ratio with another color (WCAG)
    pub fn contrast_ratio(&self, other: &RgbColor) -> f64 {
        let l1 = self.luminance();
        let l2 = other.luminance();
        let (lighter, darker) = if l1 > l2 { (l1, l2) } else { (l2, l1) };
        (lighter + 0.05) / (darker + 0.05)
    }
}

/// Theme support utilities
#[derive(Debug, Default)]
pub struct ThemeSupport {
    /// Current dark mode state
    pub dark_mode: bool,
    /// Color scheme preference
    pub preference: ColorSchemePreference,
}

impl ThemeSupport {
    /// Create new theme support
    pub fn new() -> Self { Self::default() }

    /// Resolve effective dark mode
    pub fn resolve_mode(&self) -> bool {
        match self.preference {
            ColorSchemePreference::ForceDark => true,
            ColorSchemePreference::ForceLight => false,
            ColorSchemePreference::Default => self.dark_mode,
        }
    }

    /// Legacy resolve (static)
    pub fn resolve_mode_static(pref: ColorSchemePreference, system_dark: bool) -> bool {
        match pref {
            ColorSchemePreference::ForceDark => true,
            ColorSchemePreference::ForceLight => false,
            ColorSchemePreference::Default => system_dark,
        }
    }

    /// Set system dark mode
    pub fn set_system_dark(&mut self, dark: bool) {
        self.dark_mode = dark;
    }

    /// Set preference
    pub fn set_preference(&mut self, pref: ColorSchemePreference) {
        self.preference = pref;
    }

    /// Check if should be dark
    pub fn is_dark(&self) -> bool {
        self.resolve_mode()
    }

    /// Get foreground color for background (auto contrast)
    pub fn auto_foreground(&self, bg_hex: &str) -> Option<String> {
        let bg = RgbColor::from_hex(bg_hex)?;
        let fg = if bg.is_dark() {
            RgbColor { r: 0xFF, g: 0xFF, b: 0xFF }
        } else {
            RgbColor { r: 0x1A, g: 0x1A, b: 0x2E }
        };
        Some(fg.to_hex())
    }

    /// Check WCAG contrast compliance (AA = 4.5:1)
    pub fn check_contrast(&self, fg_hex: &str, bg_hex: &str) -> Option<f64> {
        let fg = RgbColor::from_hex(fg_hex)?;
        let bg = RgbColor::from_hex(bg_hex)?;
        Some(fg.contrast_ratio(&bg))
    }

    /// Check if contrast meets AA standard
    pub fn meets_aa(&self, fg_hex: &str, bg_hex: &str) -> bool {
        self.check_contrast(fg_hex, bg_hex)
            .map(|r| r >= 4.5)
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_scheme_preference() {
        let prefs = vec![
            ColorSchemePreference::Default,
            ColorSchemePreference::ForceDark,
            ColorSchemePreference::ForceLight,
        ];
        assert_eq!(prefs.len(), 3);
    }

    #[test]
    fn test_rgb_from_hex() {
        let c = RgbColor::from_hex("#FF8000").unwrap();
        assert_eq!(c.r, 0xFF);
        assert_eq!(c.g, 0x80);
        assert_eq!(c.b, 0x00);
        assert_eq!(c.to_hex(), "#FF8000");
    }

    #[test]
    fn test_rgb_from_hex_invalid() {
        assert!(RgbColor::from_hex("#GG0000").is_none());
        assert!(RgbColor::from_hex("#FFF").is_none()); // too short
        assert!(RgbColor::from_hex("").is_none());
    }

    #[test]
    fn test_is_dark() {
        let dark = RgbColor::from_hex("#000000").unwrap();
        let light = RgbColor::from_hex("#FFFFFF").unwrap();
        assert!(dark.is_dark());
        assert!(!light.is_dark());
    }

    #[test]
    fn test_contrast_ratio() {
        let black = RgbColor::from_hex("#000000").unwrap();
        let white = RgbColor::from_hex("#FFFFFF").unwrap();
        let ratio = black.contrast_ratio(&white);
        assert!(ratio > 20.0); // ~21:1
        assert_eq!(black.contrast_ratio(&black), 1.0);
    }

    #[test]
    fn test_resolve_mode() {
        let mut ts = ThemeSupport::new();
        ts.set_system_dark(true);
        assert!(ts.is_dark());
        
        ts.set_preference(ColorSchemePreference::ForceLight);
        assert!(!ts.is_dark());
        
        ts.set_preference(ColorSchemePreference::ForceDark);
        assert!(ts.is_dark());
    }

    #[test]
    fn test_auto_foreground() {
        let ts = ThemeSupport::new();
        let fg_dark_bg = ts.auto_foreground("#000000").unwrap();
        assert_eq!(fg_dark_bg, "#FFFFFF"); // light text on dark bg
        
        let fg_light_bg = ts.auto_foreground("#FFFFFF").unwrap();
        assert_eq!(fg_light_bg, "#1A1A2E"); // dark text on light bg
    }

    #[test]
    fn test_contrast_check() {
        let ts = ThemeSupport::new();
        assert!(ts.meets_aa("#FFFFFF", "#000000")); // high contrast
        assert!(!ts.meets_aa("#777777", "#888888")); // low contrast
        
        let ratio = ts.check_contrast("#FFFFFF", "#000000").unwrap();
        assert!(ratio > 20.0);
    }

    #[test]
    fn test_invalid_hex() {
        let ts = ThemeSupport::new();
        assert!(ts.auto_foreground("invalid").is_none());
        assert!(ts.check_contrast("bad", "#000").is_none());
    }
}
