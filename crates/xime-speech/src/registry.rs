//! 本地 ASR 模型注册表（对齐 Android xime speech 的 `AsrModelRegistry`）。
//!
//! 两端共用同一批 ModelScope 模型包（都是 sherpa-onnx 导出格式）：
//! 注册表是宿主侧的**权威文件角色映射**——扩展商店条目只提供下载与展示
//! 信息，按 id 在这里查到 profile 才能装载；索引未收录的内置 profile 仍
//! 可被选中使用。未知 id（索引新增、本端未随版更新）回退默认文件布局，
//! 保证按通用命名发布的 zipformer2 模型仍可加载（目录名保留原 id）。

/// 模型包的完整性锚点：字节数 + SHA-256。
///
/// 下载地址形如 `resolve/master/...`——master 是可变分支，仓库所有者随时能
/// 用同名文件换内容，URL 不变。内置摘要把「哪一次发布」钉成内容锚点：
/// 下载方必须同时比对字节数与摘要，不符即拒绝安装。
///
/// 摘要值是**发布事实**，不是缓存：填错会让所有人下载失败，改值必须走
/// `D-nnn` 记录原因（例如上游确实重打包了）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelChecksum {
    /// 小写十六进制 SHA-256（64 字符）。
    pub sha256: String,
    /// 下载包（tar.bz2）的精确字节数，先做廉价预检再比摘要。
    pub size_bytes: u64,
}

/// 一个本地流式模型的角色清单：模型目录内四个文件的相对名 + 下载信息。
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct AsrModelProfile {
    /// 模型 id（= 模型目录名，与扩展商店/宿主持久化的选中值一致）。
    pub id: String,
    /// 展示名（设置页）。
    pub name: String,
    /// 一句话描述（设置页）。
    pub description: String,
    /// 语言标签："zh" / "zh-en"。
    pub language: String,
    /// 下载包大小（如实标注，仅展示）。
    pub size: String,
    /// tar.bz2 包下载地址（ModelScope）。
    pub download_url: String,
    /// 下载包的完整性锚点；`None` 表示没有内置摘要，宿主必须拒绝下载
    /// （宁可装不上，也不装来路不明的东西）。
    pub checksum: Option<ModelChecksum>,
    /// 编码器 onnx 文件名。
    pub encoder_file: String,
    /// 解码器 onnx 文件名。
    pub decoder_file: String,
    /// 联接器 onnx 文件名。
    pub joiner_file: String,
    /// 词表文件名（X-ASR 为 5000 BPE；zipformer-zh 为字级）。
    pub tokens_file: String,
}

/// 内置模型注册表（新模型在此追加；默认模型见 [`AsrModelRegistry::default_profile`]）。
pub struct AsrModelRegistry;

impl AsrModelRegistry {
    /// X-ASR-zh-en 流式模型（480ms chunk，int8，带标点变体）。
    ///
    /// 基于 icefall/Zipformer transducer，约 100 万小时中英文数据训练，
    /// 5000 BPE 词表（▁ 剥离与 `<0xNN>` 字节回退由 sherpa-onnx 解码器处理）。
    /// 词表含标点 token：输出带中英文标点，英文大小写混合——听写文本直接可用，
    /// 因此是本端默认模型（Android 默认是 zipformer-zh-int8，两端注册表同表、
    /// 各自默认可不同）。
    ///
    /// `name` 是**给用户看的**（设置页下拉/状态行），别放模型 id 那种工程名；
    /// 技术出处写进 `description` 或注释。
    fn x_asr_480ms_zh_en_punct_int8() -> AsrModelProfile {
        AsrModelProfile {
            id: "x-asr-480ms-zh-en-punct-int8".into(),
            name: "中英混输（自动标点）".into(),
            description: "说中文、说英文都行，边说边出字，结果自动加标点；日常输入推荐这个".into(),
            language: "zh-en".into(),
            size: "133.90MB".into(),
            download_url: "https://www.modelscope.cn/models/adaada88/sherpa-onnx-x-asr-480ms-streaming-zipformer-transducer-zh-en-punct-int8/resolve/master/sherpa-onnx-x-asr-480ms-streaming-zipformer-transducer-zh-en-punct-int8-2026-06-05.tar.bz2".into(),
            // 2026-11-09 从 ModelScope 文件 API 取值，并实际下载一次核对字节与摘要。
            checksum: Some(ModelChecksum {
                sha256: "fa5f63d618e5a01526e275a358bb7772e403f84808a4769fba52cffd8160bf74".into(),
                size_bytes: 133_895_136,
            }),
            encoder_file: "encoder.int8.onnx".into(),
            decoder_file: "decoder.onnx".into(),
            joiner_file: "joiner.int8.onnx".into(),
            tokens_file: "tokens.txt".into(),
        }
    }

    /// sherpa-onnx 官方中文流式 zipformer（int8，字级词表，无标点输出）。
    fn zipformer_zh_int8() -> AsrModelProfile {
        AsrModelProfile {
            id: "zipformer-zh-int8".into(),
            name: "纯中文（不带标点）".into(),
            description: "只识别中文，输出不带标点；说的全是中文时可选它".into(),
            language: "zh".into(),
            size: "132.63MB".into(),
            download_url: "https://www.modelscope.cn/models/bikeand/asr/resolve/master/sherpa-onnx-streaming-zipformer-zh-int8-2025-06-30.tar.bz2".into(),
            // 2026-11-09 从 ModelScope 文件 API 取值，并实际下载一次核对字节与摘要。
            checksum: Some(ModelChecksum {
                sha256: "5a2832047ea1f97dd0dc595b816c230c4bafad65cfc0341fa57517cadc50afd0".into(),
                size_bytes: 132_634_597,
            }),
            encoder_file: "encoder.int8.onnx".into(),
            decoder_file: "decoder.onnx".into(),
            joiner_file: "joiner.int8.onnx".into(),
            tokens_file: "tokens.txt".into(),
        }
    }

    /// 推荐模型 id（= 首个内置模型）：设置页给这一个挂「推荐」标记。
    pub fn recommended_id() -> String {
        Self::profiles()
            .first()
            .map(|profile| profile.id.clone())
            .unwrap_or_default()
    }

    /// 全部内置模型适配（对齐 Android `AsrModelRegistry.profiles`，首项即默认）。
    pub fn profiles() -> Vec<AsrModelProfile> {
        vec![
            Self::x_asr_480ms_zh_en_punct_int8(),
            Self::zipformer_zh_int8(),
        ]
    }

    /// 默认模型：X-ASR 带标点（输入法听写要的是「直接可用的文本」）。
    pub fn default_profile() -> AsrModelProfile {
        Self::x_asr_480ms_zh_en_punct_int8()
    }

    /// 按 id 查找内置适配。
    pub fn find_by_id(id: &str) -> Option<AsrModelProfile> {
        Self::profiles().into_iter().find(|p| p.id == id)
    }

    /// 按 id 取适配；未知 id（索引新增、本端未随之更新）回退默认文件布局
    /// 但保留原 id 作目录名——按通用命名发布的 zipformer2 模型仍可加载。
    ///
    /// 回退结果**只用于装载（文件布局）**：`checksum` 与 `download_url` 一律
    /// 清空——不能把默认模型的摘要套到未知 id 上，否则会拿 A 的锚点校验 B。
    pub fn profile_or_default(id: &str) -> AsrModelProfile {
        Self::find_by_id(id).unwrap_or_else(|| {
            let mut profile = Self::default_profile();
            profile.id = id.to_string();
            profile.download_url.clear();
            profile.checksum = None;
            profile
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_contains_both_android_profiles() {
        let ids: Vec<String> = AsrModelRegistry::profiles()
            .iter()
            .map(|p| p.id.clone())
            .collect();
        assert_eq!(
            ids,
            vec!["x-asr-480ms-zh-en-punct-int8", "zipformer-zh-int8"]
        );
    }

    #[test]
    fn default_is_x_asr_punct() {
        // 与 Android 默认（zipformer-zh-int8）刻意不同：听写文本要带标点可直接上屏
        assert_eq!(
            AsrModelRegistry::default_profile().id,
            "x-asr-480ms-zh-en-punct-int8"
        );
        assert_eq!(
            AsrModelRegistry::profiles()[0].id,
            AsrModelRegistry::default_profile().id
        );
    }

    #[test]
    fn find_by_id_hits_builtin() {
        let p = AsrModelRegistry::find_by_id("zipformer-zh-int8");
        assert!(p.is_some());
        assert_eq!(p.unwrap().language, "zh");
        assert!(AsrModelRegistry::find_by_id("no-such-model").is_none());
    }

    #[test]
    fn unknown_id_falls_back_to_default_layout_with_original_id() {
        let p = AsrModelRegistry::profile_or_default("future-model-id");
        assert_eq!(p.id, "future-model-id");
        // 文件布局回退默认（X-ASR 命名），目录名用原 id
        assert_eq!(
            p.encoder_file,
            AsrModelRegistry::default_profile().encoder_file
        );
        // 已知 id 原样返回
        let known = AsrModelRegistry::profile_or_default("zipformer-zh-int8");
        assert_eq!(known.language, "zh");
    }

    #[test]
    fn builtin_profiles_carry_checksum_anchor() {
        for p in AsrModelRegistry::profiles() {
            let checksum = p
                .checksum
                .as_ref()
                .unwrap_or_else(|| panic!("{} 缺少内置校验值", p.id));
            assert_eq!(checksum.sha256.len(), 64, "{}", p.id);
            assert!(
                checksum
                    .sha256
                    .chars()
                    .all(|ch| ch.is_ascii_hexdigit() && !ch.is_ascii_uppercase()),
                "{} 摘要应为小写十六进制: {}",
                p.id,
                checksum.sha256
            );
            // 现有两个包都在 130 MB 量级；这个下界能挡住「填了个 0」的低级错误。
            assert!(
                checksum.size_bytes > 100 * 1024 * 1024,
                "{} 字节数看着不对: {}",
                p.id,
                checksum.size_bytes
            );
        }
    }

    #[test]
    fn fallback_profile_has_no_checksum_or_download_url() {
        let p = AsrModelRegistry::profile_or_default("future-model-id");
        assert!(p.checksum.is_none(), "回退不得继承默认模型的摘要");
        assert!(p.download_url.is_empty(), "回退结果不可下载");
    }

    #[test]
    fn profiles_carry_download_and_file_roles() {
        for p in AsrModelRegistry::profiles() {
            assert!(
                p.download_url.starts_with("https://www.modelscope.cn/"),
                "{} url",
                p.id
            );
            assert!(p.download_url.ends_with(".tar.bz2"), "{} archive", p.id);
            assert!(!p.encoder_file.is_empty());
            assert!(!p.decoder_file.is_empty());
            assert!(!p.joiner_file.is_empty());
            assert_eq!(p.tokens_file, "tokens.txt");
        }
    }
}
