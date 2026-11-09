use crate::theme::ThemeColors;
use iced::widget::{
    button, container, opaque, row, stack, text, toggler, toggler::Status as TogglerStatus,
    Button,
};
use iced::{border, Alignment, Background, Border, Color, Element, Length, Shadow, Vector};

/// 圆角：对齐 HeroUI 的 rounded-large / medium / small。
pub const RADIUS_LG: f32 = 14.0;
pub const RADIUS_MD: f32 = 12.0;
pub const RADIUS_SM: f32 = 8.0;

/// 卡片投影：HeroUI 的 shadow-small（浅色下几乎不可见，只给卡片一点浮起感；
/// 深色下用更高的不透明度代替——深色底上的黑色阴影基本看不出来）。
pub fn card_shadow(colors: &ThemeColors) -> Shadow {
    if colors.is_dark {
        Shadow {
            color: Color::from_rgba8(0x00, 0x00, 0x00, 0.45),
            offset: Vector::new(0.0, 1.0),
            blur_radius: 3.0,
        }
    } else {
        Shadow {
            color: Color::from_rgba8(0x18, 0x18, 0x1b, 0.06),
            offset: Vector::new(0.0, 1.0),
            blur_radius: 3.0,
        }
    }
}

/// 应用 UI 字体：显式指定微软雅黑 UI（Windows 全版本自带，拉丁+中文覆盖完整）。
/// 不用通用族名（Sans Serif）：fontdb 在 Windows 上的通用族解析可能命中
/// 图标字体（Segoe Fluent Icons 等，也被归类为 sans-serif），ASCII 会渲染成
/// 符号乱码（中文不受影响——图标字体不含 CJK，回退雅黑）。
pub const UI_FONT: iced::font::Font = iced::font::Font::with_name("Microsoft YaHei UI");

pub fn semibold() -> iced::font::Font {
    iced::font::Font {
        weight: iced::font::Weight::Semibold,
        ..UI_FONT
    }
}

/// 中等字重：小字号正文在 1x 显示器上补偿缺失的笔画加粗（CoreText
/// stem darkening），避免发虚。
pub fn medium() -> iced::font::Font {
    iced::font::Font {
        weight: iced::font::Weight::Medium,
        ..UI_FONT
    }
}

/// 居中模态弹窗：半透明遮罩 + 居中内容卡片（iced 0.14 无内置 Modal，stack 实现）。
/// `dialog` 为 None 时原样返回底页。
pub fn modal_dialog<'a, Message: 'a>(
    base: Element<'a, Message>,
    dialog: Option<Element<'a, Message>>,
    colors: &ThemeColors,
) -> Element<'a, Message> {
    modal_dialog_sized(base, dialog, colors, 520.0, None)
}

/// 固定尺寸版本：导入弹窗内容多，避免随着状态文字/进度行变化而改变窗口大小。
pub fn modal_dialog_sized<'a, Message: 'a>(
    base: Element<'a, Message>,
    dialog: Option<Element<'a, Message>>,
    colors: &ThemeColors,
    width: f32,
    height: Option<f32>,
) -> Element<'a, Message> {
    let Some(dialog) = dialog else {
        return base;
    };
    let colors = *colors;
    let mut card = container(dialog)
        .width(Length::Fixed(width))
        .max_width(width)
        .padding(18)
        .style(move |_| container::Style {
            background: Some(Background::Color(colors.surface)),
            text_color: Some(colors.foreground),
            border: Border {
                color: colors.border,
                width: 1.0,
                radius: border::radius(RADIUS_LG),
            },
            shadow: Shadow {
                color: Color::from_rgba8(0x00, 0x00, 0x00, 0.28),
                offset: Vector::new(0.0, 8.0),
                blur_radius: 24.0,
            },
            ..container::Style::default()
        });
    if let Some(height) = height {
        card = card.height(Length::Fixed(height));
    }
    // opaque：遮罩必须吃掉鼠标事件，否则点击会穿透到下面的页面。
    let overlay = opaque(
        container(card)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(|_| container::Style {
                background: Some(Background::Color(Color::BLACK.scale_alpha(0.4))),
                ..container::Style::default()
            }),
    );
    stack![base, overlay].into()
}

// ---- container styles ----

/// 卡片容器样式：圆角 + 边框 + 柔和底色 + 一层极浅投影。
pub fn card_style(colors: &ThemeColors) -> container::Style {
    container::Style {
        background: Some(Background::Color(colors.surface)),
        text_color: Some(colors.foreground),
        border: Border {
            color: colors.border,
            width: 1.0,
            radius: border::radius(RADIUS_LG),
        },
        shadow: card_shadow(colors),
        ..container::Style::default()
    }
}

/// 侧栏样式：无圆角的整块面板，与窗口底色分层。
pub fn sidebar_style(colors: &ThemeColors) -> container::Style {
    container::Style {
        background: Some(Background::Color(colors.surface)),
        text_color: Some(colors.foreground),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: border::radius(0.0),
        },
        ..container::Style::default()
    }
}

/// 滚动条样式：窄滚动条 + 半透明圆角滑块，贴合深色主题。
pub fn scroll_style(
    colors: &ThemeColors,
    theme: &iced::Theme,
    status: iced::widget::scrollable::Status,
) -> iced::widget::scrollable::Style {
    let mut style = iced::widget::scrollable::default(theme, status);
    let rail = iced::widget::scrollable::Rail {
        background: None,
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: border::radius(999.0),
        },
        scroller: iced::widget::scrollable::Scroller {
            background: Background::Color(Color {
                a: 0.35,
                ..colors.primary
            }),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: border::radius(999.0),
            },
        },
    };
    style.vertical_rail = rail;
    style.horizontal_rail = rail;
    style
}

/// 无滚动条样式：横向筛选条这类地方不需要可见滚动条——默认滚动条会盖在
/// 最后一行文字上，把「拼音」的底部压掉一截。轨道与滑块都做成透明，
/// 只用滚轮/触控板滚动。
pub fn scroll_style_bare(
    colors: &ThemeColors,
    theme: &iced::Theme,
    status: iced::widget::scrollable::Status,
) -> iced::widget::scrollable::Style {
    let mut style = scroll_style(colors, theme, status);
    let bare = iced::widget::scrollable::Rail {
        background: None,
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: border::radius(0.0),
        },
        scroller: iced::widget::scrollable::Scroller {
            background: Background::Color(Color::TRANSPARENT),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: border::radius(0.0),
            },
        },
    };
    style.vertical_rail = bare;
    style.horizontal_rail = bare;
    style
}

// ---- button styles ----

/// 主要操作按钮：主色填充 + 主色文字。
pub fn primary_button_style(colors: &ThemeColors, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered | button::Status::Pressed => colors.primary_hover,
        _ => colors.primary,
    };
    button::Style {
        background: Some(Background::Color(bg)),
        text_color: colors.on_primary,
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: border::radius(RADIUS_MD),
        },
        ..button::Style::default()
    }
}

/// 危险按钮（卸载等）：错误色填充 + 白色文字。
pub fn danger_button_style(colors: &ThemeColors, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered | button::Status::Pressed => colors.error,
        _ => colors.error,
    };
    button::Style {
        background: Some(Background::Color(bg)),
        text_color: colors.on_error,
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: border::radius(RADIUS_MD),
        },
        ..button::Style::default()
    }
}

/// 次要操作按钮（HeroUI `bordered` 变体）：浅色底 + 一像素细描边。
///
/// 不用「灰色填充」的原因：设置页的底色本身是浅灰，灰填充的按钮在页面上
/// 直接消失，只剩一排文字飘着；白底 + 细描边在页面和卡片里都立得住。
pub fn secondary_button_style(colors: &ThemeColors, status: button::Status) -> button::Style {
    let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
    button::Style {
        background: Some(Background::Color(if hovered {
            colors.surface_hover
        } else {
            colors.control_bg
        })),
        text_color: colors.foreground,
        border: Border {
            color: colors.control_border,
            width: 1.0,
            radius: border::radius(RADIUS_MD),
        },
        ..button::Style::default()
    }
}

/// 导航按钮样式：active 为主色浅底 + 主色文字；悬停为浅色底。
pub fn nav_button_style(
    colors: &ThemeColors,
    status: button::Status,
    active: bool,
) -> button::Style {
    let base = button::Style {
        background: None,
        text_color: if active {
            colors.primary
        } else {
            colors.foreground_muted
        },
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: border::radius(RADIUS_MD),
        },
        ..button::Style::default()
    };
    if active {
        button::Style {
            background: Some(Background::Color(colors.primary_dim)),
            text_color: colors.primary,
            ..base
        }
    } else {
        let bg = match status {
            button::Status::Hovered | button::Status::Pressed => {
                Some(Background::Color(colors.surface_hover))
            }
            _ => None,
        };
        button::Style {
            background: bg,
            ..base
        }
    }
}

/// 数字输入步进按钮：透明底，悬停浅色底。
pub fn number_button_style(colors: &ThemeColors, status: button::Status) -> button::Style {
    button::Style {
        background: match status {
            button::Status::Hovered | button::Status::Pressed => {
                Some(Background::Color(colors.surface_hover))
            }
            _ => None,
        },
        text_color: colors.foreground,
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: border::radius(RADIUS_SM),
        },
        ..button::Style::default()
    }
}

/// 文本输入框样式（HeroUI `flat` 变体）：浅灰填充、默认无描边，
/// 悬停加深、聚焦时主色描边。
pub fn text_input_style(
    colors: &ThemeColors,
    status: iced::widget::text_input::Status,
) -> iced::widget::text_input::Style {
    let (border_color, border_width) = match status {
        iced::widget::text_input::Status::Focused { .. } => (colors.primary, 1.5),
        iced::widget::text_input::Status::Hovered => (colors.border_strong, 1.0),
        _ => (Color::TRANSPARENT, 1.0),
    };
    iced::widget::text_input::Style {
        background: Background::Color(colors.surface_variant),
        border: Border {
            color: border_color,
            width: border_width,
            radius: border::radius(RADIUS_MD),
        },
        icon: colors.foreground_muted,
        placeholder: colors.foreground_faint,
        value: colors.foreground,
        selection: colors.selection,
    }
}

// ---- buttons ----

/// 控件统一高度：输入框、下拉框、按钮都按这一档排版。
///
/// 以前每个控件各写各的 padding（按钮 30/32/34 三种高度混在一行里），
/// 同一行左右控件会高低错开，看着像拼出来的。
pub const CONTROL_HEIGHT: f32 = 36.0;

/// 按钮内容垂直居中。
///
/// iced 的 `button` 只做「限流 + 内缩」（`layout::padded`），内容一律贴顶放；
/// 固定高度的按钮里文字会整体偏上两三像素，一排按钮看过去明显不在一条中线上。
fn vcenter<'a, Message: 'a>(content: Element<'a, Message>) -> Element<'a, Message> {
    container(content)
        .width(Length::Shrink)
        .height(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

/// 主要按钮：主色填充。
pub fn button_primary<'a, Message: Clone + 'a>(
    label: impl Into<String>,
    colors: &ThemeColors,
    on_press: Message,
) -> Button<'a, Message> {
    let colors = *colors;
    button(vcenter(
        text(label.into())
            .size(14)
            .font(medium())
            .color(colors.on_primary)
            .into(),
    ))
    .padding([0, 18])
    .height(CONTROL_HEIGHT)
    .style(move |_theme, status| primary_button_style(&colors, status))
    .on_press(on_press)
}

/// 次要按钮：浅灰填充、无描边（HeroUI flat）。
pub fn button_secondary<'a, Message: Clone + 'a>(
    label: impl Into<String>,
    colors: &ThemeColors,
    on_press: Message,
) -> Button<'a, Message> {
    let colors = *colors;
    button(vcenter(
        text(label.into())
            .size(14)
            .font(medium())
            .color(colors.foreground)
            .into(),
    ))
    .padding([0, 16])
    .height(CONTROL_HEIGHT)
    .style(move |_theme, status| secondary_button_style(&colors, status))
    .on_press(on_press)
}

/// 危险按钮：错误色填充（卸载等破坏性操作）。
pub fn button_danger<'a, Message: Clone + 'a>(
    label: impl Into<String>,
    colors: &ThemeColors,
    on_press: Message,
) -> Button<'a, Message> {
    let colors = *colors;
    button(vcenter(
        text(label.into())
            .size(14)
            .font(medium())
            .color(colors.on_error)
            .into(),
    ))
    .padding([0, 16])
    .height(CONTROL_HEIGHT)
    .style(move |_theme, status| danger_button_style(&colors, status))
    .on_press(on_press)
}

/// 禁用态按钮：进行中的操作（「安装中…」「下载中…」「卸载中…」）占位用。
/// 尺寸与次要按钮一致，只把文字压成弱化色，且不带 `on_press`（点不动）。
pub fn button_disabled<'a, Message: 'a + Clone>(
    label: impl Into<String>,
    colors: &ThemeColors,
) -> Element<'a, Message> {
    let colors = *colors;
    button(vcenter(
        text(label.into())
            .size(14)
            .color(colors.foreground_muted)
            .into(),
    ))
    .padding([0, 16])
    .height(CONTROL_HEIGHT)
    .style(move |_theme, _status| secondary_button_style(&colors, button::Status::Disabled))
    .into()
}

/// 纯文字按钮：主色文字 + 悬停浅底（用于「重试」等）。
pub fn text_button<'a, Message: Clone + 'a>(
    label: impl Into<String>,
    colors: &ThemeColors,
    on_press: Message,
) -> Button<'a, Message> {
    let colors = *colors;
    button(vcenter(
        text(label.into())
            .size(14)
            .font(medium())
            .color(colors.primary)
            .into(),
    ))
    .padding([0, 12])
    .height(CONTROL_HEIGHT)
    .style(move |_theme, status| {
        let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
        button::Style {
            background: if hovered {
                Some(Background::Color(colors.primary_dim))
            } else {
                None
            },
            text_color: colors.primary,
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: border::radius(RADIUS_MD),
            },
            ..button::Style::default()
        }
    })
    .on_press(on_press)
}

/// 中间省略的长值：UUID / 路径这类东西缩短后仍要能认出两头，
/// 直接截尾会把最有信息量的部分（设备尾号、末级目录）切掉。
pub fn shorten_middle(value: &str, max_chars: usize) -> String {
    let chars: Vec<char> = value.chars().collect();
    if chars.len() <= max_chars || max_chars < 8 {
        return value.to_string();
    }
    let head = max_chars / 2;
    let tail = max_chars - head - 1;
    let mut out: String = chars[..head].iter().collect();
    out.push('…');
    out.extend(&chars[chars.len() - tail..]);
    out
}

/// 只读值文字（右对齐、弱化色）：给「本机标识 / 目录」这类行用，
/// 长度先经 `shorten_middle` 压过，撑不破卡片。
pub fn value_label<'a, Message: 'a>(
    value: impl Into<String>,
    limit: usize,
    colors: &ThemeColors,
) -> Element<'a, Message> {
    let value = shorten_middle(&value.into(), limit);
    text(value).size(13).color(colors.foreground_muted).into()
}

// ---- labels / kbd ----

/// 普通文本标签。
pub fn label<'a, Message: 'a>(text_: &'a str, colors: &ThemeColors) -> Element<'a, Message> {
    text(text_.to_string())
        .size(14)
        .font(medium())
        .color(colors.foreground)
        .into()
}

/// 键盘按键样式（kbd）。
pub fn kbd<'a, Message: 'a>(key: &'a str, colors: &ThemeColors) -> Element<'a, Message> {
    let colors = *colors;
    container(
        text(key.to_string())
            .size(13)
            .color(colors.foreground_muted),
    )
    .padding([3, 8])
    .style(move |_| container::Style {
        background: Some(Background::Color(colors.surface_variant)),
        border: Border {
            color: colors.border,
            width: 1.0,
            radius: border::radius(6.0),
        },
        ..container::Style::default()
    })
    .into()
}

/// 状态胶囊：圆角标签。
pub fn badge<'a, Message: 'a>(label: &'a str, fg: Color, bg: Color) -> Element<'a, Message> {
    container(text(label.to_string()).size(12).color(fg))
        .padding([3, 10])
        .style(move |_| container::Style {
            background: Some(Background::Color(bg)),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: border::radius(999.0),
            },
            ..container::Style::default()
        })
        .into()
}

// ---- progress ----

/// 细进度条（下载进度等）：轨道用底色，条用主色，圆角。
///
/// iced 自带的 progress_bar 样式在深色主题下跟卡片底色糊在一起，这里显式
/// 指定三段（轨道 / 条 / 圆角），粗细压到 6px——它是卡片里的**一行信息**，
/// 不该抢标题的视觉权重。
pub fn progress<'a, Message: 'a>(value: f32, colors: &ThemeColors) -> Element<'a, Message> {
    let colors = *colors;
    iced::widget::progress_bar(0.0..=1.0, value.clamp(0.0, 1.0))
        // ProgressBar 的尺寸走 length（长度）/ girth（粗细），不是 width/height。
        .length(Length::Fill)
        .girth(Length::Fixed(6.0))
        .style(move |_theme| iced::widget::progress_bar::Style {
            background: Background::Color(colors.surface_variant),
            bar: Background::Color(colors.primary),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: border::radius(999.0),
            },
        })
        .into()
}

// ---- pick_list ----

/// 下拉选择框样式（HeroUI flat）：浅灰填充、默认无描边，展开时主色描边。
///
/// iced 默认 pick_list 用它的主题色，和本应用的 `ThemeColors`（自定义深/浅色）
/// 不同源，混在卡片里会显得"借来的控件"，所以统一到这里描一遍。
pub fn pick_list_style(
    colors: &ThemeColors,
    status: iced::widget::pick_list::Status,
) -> iced::widget::pick_list::Style {
    let border_color = match status {
        iced::widget::pick_list::Status::Opened { .. } => colors.primary,
        iced::widget::pick_list::Status::Hovered => colors.border_strong,
        iced::widget::pick_list::Status::Active => Color::TRANSPARENT,
    };
    iced::widget::pick_list::Style {
        text_color: colors.foreground,
        background: Background::Color(colors.surface_variant),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: border::radius(RADIUS_MD),
        },
        handle_color: colors.foreground_muted,
        placeholder_color: colors.foreground_muted,
    }
}

// ---- switch ----

/// 开关（toggler），样式贴合主题。
pub fn switch<'a, Message: Clone + 'a>(
    checked: bool,
    colors: &ThemeColors,
    on_toggle: impl Fn(bool) -> Message + 'a,
) -> Element<'a, Message> {
    let colors = *colors;
    toggler(checked)
        .on_toggle(on_toggle)
        .style(move |_theme, _status: TogglerStatus| toggler::Style {
            background: Background::Color(if checked {
                colors.primary
            } else {
                colors.border_strong
            }),
            background_border_width: 0.0,
            background_border_color: Color::TRANSPARENT,
            foreground: Background::Color(colors.on_primary),
            foreground_border_width: 0.0,
            foreground_border_color: Color::TRANSPARENT,
            text_color: Some(colors.foreground),
            border_radius: Some(border::radius(999.0)),
            padding_ratio: 0.15,
        })
        .into()
}

// ---- number input ----

/// 数字输入框：− 值 + 三段式步进控件。
pub fn number_input<'a, Message: Clone + 'a>(
    value: f64,
    min: f64,
    max: f64,
    step: f64,
    colors: &ThemeColors,
    on_change: impl Fn(f64) -> Message + 'a,
) -> Element<'a, Message> {
    let colors = *colors;
    let can_dec = value > min;
    let can_inc = value < max;

    let dec_btn = button(text("−").size(16).color(if can_dec {
        colors.foreground
    } else {
        colors.disabled
    }))
    .padding([5, 12])
    .style(move |_theme, status| number_button_style(&colors, status));
    let dec_btn = if can_dec {
        dec_btn.on_press(on_change((value - step).max(min)))
    } else {
        dec_btn
    };

    let inc_btn = button(text("+").size(16).color(if can_inc {
        colors.foreground
    } else {
        colors.disabled
    }))
    .padding([5, 12])
    .style(move |_theme, status| number_button_style(&colors, status));
    let inc_btn = if can_inc {
        inc_btn.on_press(on_change((value + step).min(max)))
    } else {
        inc_btn
    };

    let center = container(
        text(format!("{}", value as i32))
            .size(14)
            .color(colors.primary)
            .font(semibold()),
    )
    .width(Length::Fill)
    .center_x(Length::Fill)
    .center_y(Length::Fill);

    container(
        row![dec_btn, center, inc_btn]
            .width(Length::Fill)
            .padding(2)
            .align_y(Alignment::Center),
    )
    .width(140)
    .height(CONTROL_HEIGHT)
    .style(move |_| container::Style {
        background: Some(Background::Color(colors.surface_variant)),
        border: Border {
            color: colors.border,
            width: 1.0,
            radius: border::radius(RADIUS_MD),
        },
        ..container::Style::default()
    })
    .into()
}

/// 分段标签栏：页内切换或二选一的分支（扩展商店、剪贴板、设备关联共用）。
///
/// 只收 `labels` + 当前项 + 消息构造器，样式不对外开放——这类控件一旦各页各写
/// 一份，圆角或内边距就会一点点漂开，设置页立刻显出拼装感。
pub fn tab_bar<'a, Message: Clone + 'a>(
    labels: &[&'a str],
    active: usize,
    colors: &ThemeColors,
    on_select: impl Fn(usize) -> Message,
) -> Element<'a, Message> {
    let colors = *colors;
    let mut bar = row![].spacing(2).padding(3);

    // 等宽标签：各段按最长的那条文字取宽，否则「关闭 / 同步插件 / 设备互传」
    // 会按内容各占一块，整条控制看起来歪歪扭扭。
    let cell = labels
        .iter()
        .map(|label_text| label_text.chars().count())
        .max()
        .unwrap_or(1) as f32
        * 14.0
        + 32.0;

    for (i, label_text) in labels.iter().enumerate() {
        let is_active = i == active;
        let label_text = *label_text;
        bar = bar.push(
            button(vcenter(
                text(label_text)
                    .size(14)
                    .font(medium())
                    .color(if is_active {
                        colors.primary
                    } else {
                        colors.foreground_muted
                    })
                    .into(),
            ))
            .padding([0, 14])
            .width(Length::Fixed(cell))
            .height(CONTROL_HEIGHT - 6.0)
            .style(move |_theme, status| {
                let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
                // 选中项是一枚「药丸」：实心底色 + 一点主色；未选中项只有悬停
                // 才浮出半透明白。轨道本身比页面深一档，两者才分得开。
                let background = if is_active {
                    Some(Background::Color(colors.tab_active))
                } else if hovered {
                    Some(Background::Color(Color {
                        a: 0.55,
                        ..colors.surface
                    }))
                } else {
                    None
                };
                button::Style {
                    background,
                    text_color: if is_active {
                        colors.primary
                    } else {
                        colors.foreground_muted
                    },
                    border: Border {
                        color: Color::TRANSPARENT,
                        width: 0.0,
                        radius: border::radius(RADIUS_SM),
                    },
                    ..button::Style::default()
                }
            })
            .on_press(on_select(i)),
        );
    }

    container(bar)
        .width(Length::Shrink)
        .style(move |_| container::Style {
            background: Some(Background::Color(colors.track)),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: border::radius(RADIUS_MD),
            },
            ..container::Style::default()
        })
        .into()
}
