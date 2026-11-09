//! 命令行参数 → 应用元数据。
//!
//! 设置应用在 Windows / Linux / macOS 三平台共用同一份 UI，产品差异（名称、
//! 标识、主页、版本）由宿主传参决定。macOS 侧直接在 Rust 里调
//! [`crate::run_with`]；Windows / Linux 的打包脚本则通过这里的命令行参数描述
//! 本产品，不必为每个产品重新编一份界面。

use xime_config::AppMetadata;

/// `--help` 输出。
pub const HELP: &str = "\
用法: xime-setup [选项]

不写任何选项时使用 Xime 默认值。可用选项：
  --display-name <名称>    产品显示名，如「曦码·逸输入法」
  --settings-name <标题>   设置窗口标题，缺省为「<显示名> 设置」
  --subtitle <副标题>      侧栏品牌区副标题，默认「输入法设置」
  --tagline <简介>         关于页一句话简介
  --homepage <URL>         关于页「项目主页」打开的地址
  --version <版本>         版本号
  --app-id <标识>          librime app name，如 rime.ximeyi.setup
  --config-dir <目录名>    配置目录名，如 ximeyi
  --config-base <基名>     配置文件基名，如 ximeyi
  --distribution <名称>    librime distribution name
  --help                   显示这段说明
";

/// 把 `--k v` 形式的一串参数解析成元数据。
///
/// 解析结果里的字符串会 `Box::leak` 成 `&'static str`——设置应用进程只启动
/// 一次、元数据全程只读，这点泄漏换掉一堆生命周期噪音是划算的。
pub fn metadata_from_args<I>(args: I) -> Result<AppMetadata, String>
where
    I: IntoIterator<Item = String>,
{
    let args: Vec<String> = args.into_iter().collect();
    let mut meta = AppMetadata::default();
    let mut display_name: Option<String> = None;
    let mut settings_name: Option<String> = None;
    let mut subtitle: Option<String> = None;
    let mut tagline: Option<String> = None;
    let mut homepage: Option<String> = None;
    let mut version: Option<String> = None;
    let mut app_id: Option<String> = None;
    let mut config_dir: Option<String> = None;
    let mut config_base: Option<String> = None;
    let mut distribution: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        let flag = args[i].as_str();
        // `--help` 由调用方先处理，这里遇到说明用法传错了。
        let value = args
            .get(i + 1)
            .cloned()
            .ok_or_else(|| format!("参数 {flag} 缺少取值"))?;
        let slot = match flag {
            "--display-name" => &mut display_name,
            "--settings-name" => &mut settings_name,
            "--subtitle" => &mut subtitle,
            "--tagline" => &mut tagline,
            "--homepage" => &mut homepage,
            "--version" => &mut version,
            "--app-id" => &mut app_id,
            "--config-dir" => &mut config_dir,
            "--config-base" => &mut config_base,
            "--distribution" => &mut distribution,
            other => return Err(format!("未知参数 {other}")),
        };
        *slot = Some(value);
        i += 2;
    }

    if let Some(v) = display_name {
        meta.display_name = leak(v);
        // 标题缺省跟随显示名：中文产品名后面不带空格。
        meta.settings_name = leak(format!("{} 设置", meta.display_name));
    }
    if let Some(v) = settings_name {
        meta.settings_name = leak(v);
    }
    if let Some(v) = subtitle {
        meta.subtitle = leak(v);
    }
    if let Some(v) = tagline {
        meta.tagline = leak(v);
    }
    if let Some(v) = homepage {
        meta.homepage = leak(v);
    }
    if let Some(v) = version {
        meta.version = leak(v);
    }
    if let Some(v) = app_id {
        meta.app_name = leak(v);
    }
    if let Some(v) = config_dir {
        meta.config_dir_name = leak(v);
    }
    if let Some(v) = config_base {
        meta.config_file_base = leak(v);
    }
    if let Some(v) = distribution {
        meta.distribution_name = leak(v.clone());
        meta.distribution_code_name = leak(v);
    }
    Ok(meta)
}

fn leak(value: String) -> &'static str {
    Box::leak(value.into_boxed_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn empty_args_keeps_xime_defaults() {
        let meta = metadata_from_args(Vec::new()).unwrap();
        assert_eq!(meta.display_name, "Xime");
        assert_eq!(meta.settings_name, "Xime 设置");
    }

    #[test]
    fn display_name_derives_settings_title() {
        let meta = metadata_from_args(args(&["--display-name", "曦码·逸输入法"])).unwrap();
        assert_eq!(meta.display_name, "曦码·逸输入法");
        assert_eq!(meta.settings_name, "曦码·逸输入法 设置");
        // 未传的项保持默认。
        assert_eq!(meta.subtitle, "输入法设置");
    }

    #[test]
    fn explicit_settings_name_wins_over_derived() {
        let meta = metadata_from_args(args(&[
            "--display-name",
            "曦码·逸输入法",
            "--settings-name",
            "曦码·逸输入法设置",
        ]))
        .unwrap();
        assert_eq!(meta.settings_name, "曦码·逸输入法设置");
    }

    #[test]
    fn product_identity_options_all_apply() {
        let meta = metadata_from_args(args(&[
            "--app-id",
            "rime.ximeyi.setup",
            "--config-dir",
            "ximeyi",
            "--homepage",
            "https://github.com/ximeiorg/XimeYi",
            "--version",
            "0.1.0",
            "--distribution",
            "XimeYi",
        ]))
        .unwrap();
        assert_eq!(meta.app_name, "rime.ximeyi.setup");
        assert_eq!(meta.config_dir_name, "ximeyi");
        assert_eq!(meta.homepage, "https://github.com/ximeiorg/XimeYi");
        assert_eq!(meta.version, "0.1.0");
        assert_eq!(meta.distribution_name, "XimeYi");
        assert_eq!(meta.distribution_code_name, "XimeYi");
    }

    #[test]
    fn unknown_flag_is_rejected() {
        let err = metadata_from_args(args(&["--nope", "x"])).unwrap_err();
        assert!(err.contains("--nope"), "{err}");
    }

    #[test]
    fn missing_value_is_rejected() {
        let err = metadata_from_args(args(&["--display-name"])).unwrap_err();
        assert!(err.contains("缺少取值"), "{err}");
    }
}
