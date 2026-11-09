use iced::Color;

/// 系统主题检测（不使用任何 GUI 框架，纯平台 API）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SystemTheme {
    Light,
    Dark,
}

impl SystemTheme {
    pub fn detect() -> Self {
        #[cfg(target_os = "linux")]
        {
            if let Ok(output) = std::process::Command::new("gsettings")
                .args(["get", "org.gnome.desktop.interface", "color-scheme"])
                .output()
            {
                let value = String::from_utf8_lossy(&output.stdout);
                if value.contains("'prefer-dark'") || value.contains("'dark'") {
                    return SystemTheme::Dark;
                }
            }

            if let Ok(output) = std::process::Command::new("gsettings")
                .args(["get", "org.gnome.desktop.interface", "gtk-theme"])
                .output()
            {
                let value = String::from_utf8_lossy(&output.stdout);
                if value.contains("dark") || value.contains("Dark") {
                    return SystemTheme::Dark;
                }
            }

            if std::env::var("GTK_THEME")
                .map(|t| t.contains("dark") || t.contains("Dark"))
                .unwrap_or(false)
            {
                return SystemTheme::Dark;
            }

            if let Ok(style) = std::env::var("COLOR_SCHEME") {
                if style == "dark" || style == "Dark" {
                    return SystemTheme::Dark;
                }
            }
        }

        #[cfg(target_os = "windows")]
        {
            use windows::core::PCWSTR;
            use windows::Win32::System::Registry::*;
            unsafe {
                let mut key: u32 = 0;
                let mut size = std::mem::size_of::<u32>() as u32;
                if RegGetValueW(
                    HKEY_CURRENT_USER,
                    PCWSTR(
                        "Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize\0"
                            .encode_utf16()
                            .collect::<Vec<_>>()
                            .as_ptr(),
                    ),
                    PCWSTR(
                        "AppsUseLightTheme\0"
                            .encode_utf16()
                            .collect::<Vec<_>>()
                            .as_ptr(),
                    ),
                    RRF_RT_DWORD,
                    None,
                    Some(&mut key as *mut _ as _),
                    Some(&mut size),
                )
                .is_ok()
                {
                    if key == 0 {
                        return SystemTheme::Dark;
                    }
                }
            }
        }

        SystemTheme::Light
    }

    pub fn is_dark(&self) -> bool {
        matches!(self, SystemTheme::Dark)
    }
}

/// 颜色混合：`t=0` 时为 `a`，`t=1` 时为 `b`。
fn mix(a: Color, b: Color, t: f32) -> Color {
    Color::from_rgba(
        a.r + (b.r - a.r) * t,
        a.g + (b.g - a.g) * t,
        a.b + (b.b - a.b) * t,
        a.a + (b.a - a.a) * t,
    )
}

/// 相对亮度（BT.601）：用来决定主色上的文字该用白还是深色。
fn lightness(color: Color) -> f32 {
    0.299 * color.r + 0.587 * color.g + 0.114 * color.b
}

/// 主题颜色：Tailwind zinc 色阶 + 可定制主色，风格参考 ximed 桌面 UI。
#[derive(Clone, Copy, Debug)]
pub struct ThemeColors {
    /// 窗口底色。
    pub background: Color,
    /// 卡片 / 侧栏底色。
    pub surface: Color,
    /// 悬停底色。
    pub surface_hover: Color,
    /// 更浅的次级底色（kbd、胶囊、图标底、输入框填充）。
    pub surface_variant: Color,
    /// 分段控件（tabs）轨道底色：必须比窗口底色再深一档，否则整条轨道
    /// 和页面糊在一起，只剩下选中项在飘。
    pub track: Color,
    /// 分段控件选中项底色（白/深灰药丸）。
    pub tab_active: Color,
    /// 次要按钮底色（浮在页面和卡片上都要看得见，所以不用 surface）。
    pub control_bg: Color,
    /// 次要按钮描边。
    pub control_border: Color,
    /// 主色（强调文字、按钮填充、焦点描边）。
    pub primary: Color,
    /// 主色悬停（按钮 hover 填充）。
    pub primary_hover: Color,
    /// 主色浅底（active 导航、分段控件）。
    pub primary_dim: Color,
    /// 主色上的文字色。
    pub on_primary: Color,
    /// 次色（手写等分类图标文字色，偏青）。
    pub secondary: Color,
    /// 次色浅底（分类图标底色）。
    pub secondary_dim: Color,
    /// 第三色（表情/语音等分类图标文字色，偏橙）。
    pub tertiary: Color,
    /// 第三色浅底（分类图标底色）。
    pub tertiary_dim: Color,
    /// 主文本。
    pub foreground: Color,
    /// 次级文本。
    pub foreground_muted: Color,
    /// 弱化文本。
    pub foreground_faint: Color,
    /// 边框。
    pub border: Color,
    /// 强调边框。
    pub border_strong: Color,
    /// 禁用状态。
    pub disabled: Color,
    /// 危险色。
    pub error: Color,
    /// 危险浅底。
    pub error_dim: Color,
    /// 危险色上的文字色。
    pub on_error: Color,
    /// 成功色。
    pub success: Color,
    /// 成功浅底。
    pub success_dim: Color,
    /// 文本选区。
    pub selection: Color,
    /// 是否深色主题（阴影/叠加层需要按明暗分别取值）。
    pub is_dark: bool,
}

impl ThemeColors {
    pub fn from_theme(theme: &SystemTheme, primary_color: u32) -> Self {
        let (r, g, b) = (
            (primary_color >> 16) as u8,
            (primary_color >> 8) as u8,
            primary_color as u8,
        );
        let base = Color::from_rgb8(r, g, b);
        const WHITE: Color = Color::WHITE;
        const BLACK: Color = Color::BLACK;

        if theme.is_dark() {
            // Tailwind zinc 深色 + 主色提亮（深底可读性）
            let primary = mix(base, WHITE, 0.28);
            let primary_hover = mix(base, WHITE, 0.4);
            // 提亮后的主色可能已经偏浅，白字压不住——按亮度改用深色文字。
            let on_primary = if lightness(primary) > 0.5 {
                Color::from_rgb8(0x0a, 0x0a, 0x0c)
            } else {
                Color::WHITE
            };
            Self {
                background: Color::from_rgb8(0x09, 0x09, 0x0b),
                surface: Color::from_rgb8(0x14, 0x14, 0x17),
                surface_hover: Color::from_rgb8(0x23, 0x23, 0x27),
                surface_variant: Color::from_rgb8(0x23, 0x23, 0x27),
                track: Color::from_rgb8(0x26, 0x26, 0x2b),
                tab_active: Color::from_rgb8(0x3a, 0x3a, 0x41),
                control_bg: Color::from_rgb8(0x21, 0x21, 0x26),
                control_border: Color::from_rgb8(0x35, 0x35, 0x3c),
                primary,
                primary_hover,
                primary_dim: Color::from_rgba(primary.r, primary.g, primary.b, 0.18),
                on_primary,
                secondary: Color::from_rgb8(0x6e, 0xd4, 0xc8),
                secondary_dim: Color::from_rgba8(0x6e, 0xd4, 0xc8, 0.16),
                tertiary: Color::from_rgb8(0xf2, 0xb8, 0x7d),
                tertiary_dim: Color::from_rgba8(0xf2, 0xb8, 0x7d, 0.16),
                foreground: Color::from_rgb8(0xf4, 0xf4, 0xf5),
                foreground_muted: Color::from_rgb8(0xa1, 0xa1, 0xaa),
                foreground_faint: Color::from_rgb8(0x71, 0x71, 0x7a),
                border: Color::from_rgb8(0x29, 0x29, 0x30),
                border_strong: Color::from_rgb8(0x3f, 0x3f, 0x46),
                disabled: Color::from_rgb8(0x52, 0x52, 0x5b),
                error: Color::from_rgb8(0xfb, 0x71, 0x85),
                error_dim: Color::from_rgba8(0xfb, 0x71, 0x85, 0.15),
                on_error: Color::WHITE,
                success: Color::from_rgb8(0x34, 0xd3, 0x99),
                success_dim: Color::from_rgba8(0x34, 0xd3, 0x99, 0.15),
                selection: Color::from_rgba(base.r, base.g, base.b, 0.35),
                is_dark: true,
            }
        } else {
            // Tailwind zinc 浅色
            let primary_hover = mix(base, BLACK, 0.1);
            Self {
                background: Color::from_rgb8(0xf4, 0xf4, 0xf5),
                surface: Color::WHITE,
                surface_hover: Color::from_rgb8(0xec, 0xec, 0xef),
                surface_variant: Color::from_rgb8(0xf4, 0xf4, 0xf5),
                track: Color::from_rgb8(0xe6, 0xe6, 0xea),
                tab_active: Color::WHITE,
                control_bg: Color::WHITE,
                control_border: Color::from_rgb8(0xe3, 0xe3, 0xe8),
                primary: base,
                primary_hover,
                primary_dim: Color::from_rgba(base.r, base.g, base.b, 0.12),
                on_primary: Color::WHITE,
                secondary: Color::from_rgb8(0x14, 0x8a, 0x7d),
                secondary_dim: Color::from_rgba8(0x14, 0x8a, 0x7d, 0.12),
                tertiary: Color::from_rgb8(0xb4, 0x5e, 0x1e),
                tertiary_dim: Color::from_rgba8(0xb4, 0x5e, 0x1e, 0.12),
                foreground: Color::from_rgb8(0x18, 0x18, 0x1b),
                foreground_muted: Color::from_rgb8(0x71, 0x71, 0x7a),
                foreground_faint: Color::from_rgb8(0xa1, 0xa1, 0xaa),
                border: Color::from_rgb8(0xe7, 0xe7, 0xea),
                border_strong: Color::from_rgb8(0xd4, 0xd4, 0xd8),
                disabled: Color::from_rgb8(0xa1, 0xa1, 0xaa),
                error: Color::from_rgb8(0xe1, 0x1d, 0x48),
                error_dim: Color::from_rgba8(0xe1, 0x1d, 0x48, 0.10),
                on_error: Color::WHITE,
                success: Color::from_rgb8(0x05, 0x96, 0x69),
                success_dim: Color::from_rgba8(0x05, 0x96, 0x69, 0.10),
                selection: Color::from_rgba(base.r, base.g, base.b, 0.3),
                is_dark: false,
            }
        }
    }

    /// 生成 iced 主题（供内置控件使用：滚动条、toggler 等）。
    pub fn iced_theme(&self) -> iced::Theme {
        let palette = iced::theme::Palette {
            background: self.background,
            text: self.foreground,
            primary: self.primary,
            success: self.success,
            warning: Color::from_rgb8(0xfb, 0xb0, 0x24),
            danger: self.error,
        };
        iced::Theme::custom(
            xime_config::app_metadata().config_dir_name.to_string(),
            palette,
        )
    }
}
