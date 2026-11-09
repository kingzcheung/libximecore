# 跨设备剪贴板 P2P 协议草案

- 状态：**v0.2 —— 协议核心已实现（独立仓 `libximep2p`），发现层与端上接入待做，见 §14**
- 适用：macOS / Windows / Linux（Rust，直接依赖）+ Android（手写 JNI，见 §13）
- 归属：`libximecore`

## 1. 目标与范围

在**不使用任何第三方服务器**的前提下，让同一局域网内的两台设备互相同步剪贴板。

范围内：

- 局域网设备发现
- 动态码配对（一端显示、另一端输入）
- 端到端加密的直连通道
- 当前剪贴板条目的双向同步

本协议作为**第三种传输**与现有 WebDAV、ximed 并列，不替换、不改变其行为（见 §2.2）。

范围外（本版不做）：

- NAT 穿透、跨网络同步
- 剪贴板**历史**同步（仅同步"当前值"，与现有 `clipboard/current.json` 语义一致）
- 文件 / 图片附件的分块传输（`Profile` 已含 `data` 字段，预留但本版不启用）
- 多设备（三台及以上）的向量时钟

## 2. 复用边界与并存策略

### 2.1 复用边界

本协议**不新建同步模型**，只新增一个传输实现。以下能力已存在，直接复用：

| 能力 | 位置 | 复用方式 |
|---|---|---|
| 条目模型 `Profile` | `xime-sync-domain::profile` | 原样作为 payload |
| 幂等写入 | `ClipboardService::put()` | 返回 `PutOutcome::Unchanged` 即已去重 |
| 事件序号 | `ClipboardEvent { seq, profile }` | 通道内补发用 |
| 存储 | `ClipboardRepo` + `Storage` | 不变 |

`Profile.source` 语义已是设备标识（文档示例即 `"device-a"`），**直接用作 origin**，不新增字段。

### 2.2 与现有传输并存，不替换

现有两种传输（WebDAV 插件、ximed 自建服务器）**保持原样可用**。P2P 是第三种，三者互斥选用，不叠加。

理由是现有代码已经把传输层抽干净了，接入点只有一处：

| 端 | 接入点 | P2P 的接法 |
|---|---|---|
| Android | `ClipboardSyncPlugin` 接口（`push` / `pull` / `testConnection`，共 3 个方法） | 新增 `P2pClipboardPlugin` 实现同一接口，与 WebDAV / ximed 插件并列注册 |
| 桌面端 | `clipboard_sync.rs::ActiveSync` 的 `runtime: PluginRuntime`（具体类型） | 改为 `enum Backend { Plugin, P2p }`，`tick()` 内改调 `backend.push()` |

两侧的共同前提是：**`ClipboardSyncBridge`（Android）与 `ActiveSync` 的去重、回声抑制、退避、`pullOnce` 时机全部不变**，P2P 只实现"把一条 `Profile` 送出去 / 取回来"。

Android 侧实现示例（语义与现有插件逐一对应）：

```kotlin
override suspend fun push(profile: ClipboardProfile): Boolean =
    P2pNative.push(profile.toJson())          // 对应 clipboard_push

override suspend fun pull(): ClipboardProfile? =
    P2pNative.poll().let { ClipboardProfile.fromJson(it) }  // 无变更返回 null

override suspend fun testConnection(): String? =
    if (P2pNative.hasPeer()) null else "未连接到设备"        // null 表示正常
```

第一版 `supportsAttachments = false`。Bridge 已有该开关，未声明能力的插件**自动跳过图片同步、文本不受影响**，因此图片降级不需要额外代码。

### 2.3 配置兼容（必须遵守）

⚠️ 这是本方案唯一会造成静默故障的地方。

桌面端现有读取逻辑是：

```rust
let Ok(content) = std::fs::read_to_string(path) else { return default };
toml::from_str(&content).unwrap_or_default()   // 解析失败 → 静默回到默认
```

如果给 `SyncConfig` 增加一个**必填**的 `transport` 字段，所有旧配置会解析失败、回落到默认值，用户的同步被悄悄关掉，日志里没有任何提示。因此：

```rust
#[serde(default = "default_transport")]
transport: Transport,   // 返回 Transport::Plugin

fn default_transport() -> Transport { Transport::Plugin }
```

Android 侧同理：新字段必须有默认值，且默认值指向用户当前正在用的传输方式，保证升级后行为零变化。

### 2.4 互斥、回滚与端口

| 项 | 约定 |
|---|---|
| 同时启用多种传输 | **禁止**。剪贴板是单通道语义，双写会让同一条内容被推两次，远端回写时还可能互相触发。 |
| 互斥的实现方式 | Android 的 Bridge 只持有一个 `plugin` 实例，天然互斥；桌面端由 `enum Backend` 在编译期保证只有一个。 |
| UI 形态 | 三段选择器：`关闭 / 同步插件 / 局域网直连`（`SyncMode`）。选「同步插件」后再由下拉选具体插件（WebDAV、ximed 等同为插件，不再单列）。`enabled` 与 `transport` 一次写盘，避免 IME 热加载时读到「已启用但方式还是旧的」的瞬时态。 |
| 端口 | ximed 默认 `0.0.0.0:8443`，本协议用 47821 / 47822，**不冲突**。 |
| 回滚 | P2P 出问题时用户切回 WebDAV / ximed 即可，无需降级版本。旧版本客户端读不到 `transport` 字段时按插件模式运行，与新版互通不受影响。 |

## 3. 角色与术语

| 术语 | 含义 |
|---|---|
| Display | 配对时显示动态码的一端 |
| Input | 配对时输入动态码的一端 |
| `did` | 设备标识，16 字节，`SHA256(pub_ed)[..16]`，**由公钥派生而非独立存储** |
| 长期密钥 | Ed25519（身份签名）+ X25519（Noise IK 静态密钥），由同一 32 字节种子经 HKDF 以不同 `info` 分离派生 |
| 会话密钥 | 每次连接重新协商，不落盘 |

Display/Input 只是配对时的角色，**配对完成后两端对等**，任一端都可发起连接。

## 4. 传输层

### 4.1 端口

| 用途 | 端口 | 说明 |
|---|---|---|
| TCP 会话 | 47821 | 固定值；被占用时顺延并在 TXT 记录中声明实际值 |
| UDP 发现 | 47822 | 广播兜底 |
| TCP 配对 | 47823 | 配对专用，与同步端口分离 |

配对端口单独存在的理由：配对要独占一条连接直到双方交换完身份，而同步端口的
accept 循环必须先判断「这是已配对设备的 Noise 握手，还是新设备的配对 HELLO」。
两条流程对首帧的期待不同（Noise vs HELLO），混在一起就得在 accept 处做首帧分流，
容易出错且难以测试。分开后两条路径互不影响，代价只是多一个端口号。

### 4.2 帧格式

所有帧都是「9 字节头 + payload」：

```
offset  size  field
0       2     magic     0x58 0x50（ASCII "XP"）
2       1     version   0x01
3       1     type      见 §8.1
4       1     flags     bit0 = ENCRYPTED（payload 为 Noise 密文）
5       4     length    big-endian u32，payload 字节数
9       N     payload
```

限制：

- `length` 上限 1 MiB（1048576），超限立即断开
- 连接建立后 5 秒内未完成握手 → 断开
- 空闲 30 秒 → 断开
- 单端并发未认证连接上限 4

明文阶段的 payload 是 UTF-8 JSON；加密阶段整个 payload 是 Noise transport 密文，解密后的内容同样以 `type` 开头。

**加密 payload 的分片编码**：Noise 单条传输消息上限 65535 字节（含 16 字节 tag），而剪贴板文本可达 256 KiB，所以 payload 是分片序列：

```text
payload = (u16 大端 分片长度 || 分片密文) * N
```

- 单个分片的明文上限 60000 字节
- 空明文编码为**零条分片**（payload 长度为 0），解密结果为空
- 解密方按长度前缀逐片解；前缀不足 2 字节、长度越界、长度 < 16 一律整帧拒绝

### 4.3 示例

PING 帧（payload 为 `{}`）：

```
58 50 01 0A 00 00 00 00 02 7B 7D
```

ERROR 帧（payload 为 `{"code":2}`）：

```
58 50 01 0C 00 00 00 00 0A 7B 22 63 6F 64 65 22 3A 32 7D
```

## 5. 发现

> **当前实现状态**：只做了 §5.2 的 UDP 广播（受限广播 `255.255.255.255`），
> mDNS 与「逐网卡定向广播」尚未实现。已配对设备的重连优先走「上次地址」，
> 广播用于发现地址变化后的对端。这个组合在家庭单网段环境够用；
> 多网卡（Docker / VPN）与 AP 隔离场景的可靠性取决于后续补的两项。

### 5.1 mDNS（首选）

服务类型：`_xime-clip._tcp.local.`

TXT 记录：

| 键 | 值 |
|---|---|
| `v` | 协议版本，`"1"` |
| `did` | 设备标识，32 位小写 hex |
| `pt` | TCP 端口 |

匹配一律使用 `did`，**不得依赖主机名**（各平台会自动改名，如 `MacBook-Pro-2.local`）。

### 5.2 UDP 广播（兜底）

向 `255.255.255.255:47822` 及每个接口的子网定向广播地址发送：

```json
{"s":"xime-clip","v":1,"did":"<32 hex>","pt":47821}
```

收到广播的一端若已配对，可在同一端口单播回应自身公告。

### 5.3 降级顺序

```
mDNS → UDP 广播 → 直连上次地址 → 提示排查
```

「上次地址」在每次成功连接后写回已配对表。全部失败时提示「请确认两台设备在同一 WiFi，且路由器未开启 AP 隔离」，**不得要求用户填写 IP**。

### 5.4 实现注意

- Android 必须持有 `WifiManager.MulticastLock`，否则收不到组播
- 主机多网卡（Docker / VMware / Hyper-V / VPN）时须遍历全部接口，否则会广播到虚拟网段

## 6. 配对

### 6.1 动态码

- 6 位十进制数字，展示时按 `482 719` 分组
- 有效期 60 秒，倒计时结束立即作废并生成新码
- **码只经人眼与手指传递，不得出现在任何网络报文中**

这条是硬约束。若在公告或握手里携带 `hash(code)`，同网段攻击者可离线枚举 10^6 空间，60 秒内必然破解，届时加密形同虚设。

### 6.2 密钥协商：SPAKE2

低熵码必须经 PAKE 强化，不能直接作为 KDF 输入。选型：RustCrypto `spake2`（Ed25519 群），密码为 6 位码。

配对消息序列：

```
Input (B)                              Display (A)
    |                                      |
    |--- HELLO {ver,did,nonce_b} --------->|
    |<-- HELLO_ACK {ver,did,nonce_a,ok} ---|
    |--- SPAKE2_A {msg_b} ---------------->|
    |<-- SPAKE2_B {msg_a} -----------------|
    |--- PAIR_CONFIRM {hmac} ------------->|   HMAC(K, "c2s" || nonce_a || nonce_b)
    |<-- PAIR_OK {hmac} -------------------|   HMAC(K, "s2c" || nonce_a || nonce_b)
    |--- IDENTITY {pub_ed,pub_x25519,name}|   使用 K 派生的 AEAD 加密
    |<-- IDENTITY {pub_ed,pub_x25519,name}|
```

要点：

- 双方 `finish()` 后得到同一 32 字节 `K`
- `PAIR_CONFIRM` / `PAIR_OK` 的方向必须分离，防止反射
- `IDENTITY` 必须加密，否则设备名与公钥对旁观者可见
- 配对成功后双方写入已配对表：`did`、`pub_ed`、`pub_x25519`、`name`、`last_addr`

### 6.2.1 精确定义（两侧必须逐字节一致）

SPAKE2 参数：

| 项 | 值 |
|---|---|
| 群 | Ed25519（RustCrypto `spake2` 的 `Ed25519Group`） |
| 密码 `pwd` | 6 位码的零填充 ASCII，如 `482719`（不含空格） |
| 身份 `id_a` | Display 的 `did` hex（32 字符） |
| 身份 `id_b` | Input 的 `did` hex |
| 角色 | Display = A（`start_a`），Input = B（`start_b`）；两者传入的身份顺序都是 `(id_a, id_b)` |

密钥派生（RFC 5869，SHA-256）：

```text
salt  = nonce_a || nonce_b                  # 16 + 16 字节，顺序固定，不可交换
PRK   = HKDF-Extract(salt, IKM = K)         # 等价于 HMAC-SHA256(salt, K)
k_c2s = HKDF-Expand(PRK, "c2s",      32)
k_s2c = HKDF-Expand(PRK, "s2c",      32)
k_id  = HKDF-Expand(PRK, "identity", 32)
```

确认值：

```text
PAIR_CONFIRM = HMAC-SHA256(k_c2s, "xime-p2p/confirm")    # Input  → Display
PAIR_OK      = HMAC-SHA256(k_s2c, "xime-p2p/confirm")    # Display → Input
```

方向分离靠**不同密钥**，不靠不同消息内容。因此把 `PAIR_OK` 的值当作 `PAIR_CONFIRM` 提交必然失败。

`IDENTITY` 的 AEAD：ChaCha20-Poly1305，密钥 `k_id`，12 字节全零 nonce（`k_id` 一次配对只用一次），
AAD 为：

```text
"xime-p2p/identity/v1" || did_display(16) || did_input(16)
```

`IDENTITY` 内的 `pub_ed` 必须能派生出与 `HELLO` / `HELLO_ACK` 中声明一致的 `did`，否则整次配对失败。

### 6.3 失败处理

| 情况 | 处理 |
|---|---|
| 码错误 | 计数 +1，回 `ERROR{code:2}` 并断开 |
| 累计 3 次错误 | 立即作废当前码，Display 侧必须重新生成 |
| 超时（60 秒） | 作废，退出配对模式 |
| 已有配对进行中 | 回 `ERROR{code:4}`（BUSY） |

3 次上限 + 60 秒窗口，使在线猜测成功率不高于 `3/10^6`。

### 6.4 已配对后

配对完成后连接**不中断**，直接进入会话状态（密钥来自 SPAKE2）。后续重连走 §7。

## 7. 会话（已配对重连）

使用 `Noise_IK_25519_ChaChaPoly_BLAKE2s`：

- Initiator 为主动连接方，Responder 为监听方
- Responder 静态公钥 = 配对时记录的 `pub_x25519`
- 1-RTT 完成，Initiator 身份对被动观察者隐藏

`prologue` 固定为（共 51 字节）：

```text
"xime-p2p/session/v1" || did_lo(16) || did_hi(16)     # did 按字节序升序排列
```

两条握手消息都用 `NOISE`（0x0E）帧承载，payload 为 `{"msg":"<base64 原始 Noise 消息>"}`：
Initiator 发 msg1 → Responder 回 msg2 → 双方 `into_transport_mode()`。

**Responder 无先验信息**：Initiator 的静态公钥只有解完 msg1 才拿得到，而 prologue 需要双方 `did`。
因此 Responder 只能对已配对表**逐项试解** msg1（每项新建一个 HandshakeState，失败即换下一个），命中即停。
表项上限 8，全部失败回 `ERROR{code:3}` 并断开。个人设备表通常 1–3 项，代价可忽略。

Noise 握手完成后，其 transport 密钥即为本会话的加密密钥，此后所有帧的 `flags.bit0 = 1`，
payload 按 §4.2 的分片格式编码。

## 8. 应用消息

### 8.1 类型表

| 值 | 名称 | 阶段 | payload |
|---|---|---|---|
| 0x01 | HELLO | 配对 | `{ver,did,nonce}` |
| 0x02 | HELLO_ACK | 配对 | `{ver,did,nonce,ok}` |
| 0x03 | SPAKE2_A | 配对 | `{msg}` base64 |
| 0x04 | SPAKE2_B | 配对 | `{msg}` base64 |
| 0x05 | PAIR_CONFIRM | 配对 | `{hmac}` base64 |
| 0x06 | PAIR_OK | 配对 | `{hmac}` base64 |
| 0x07 | IDENTITY | 配对 | 密文 `{pub_ed,pub_x25519,name}` |
| 0x08 | CLIPBOARD_SET | 会话 | `{profile,seq}` |
| 0x09 | CLIPBOARD_CHANGED | 会话 | `{profile,seq}` |
| 0x0A | PING | 会话 | `{}` |
| 0x0B | PONG | 会话 | `{}` |
| 0x0C | ERROR | 任意 | `{code,msg}` |
| 0x0D | BYE | 任意 | `{reason}` |
| 0x0E | NOISE | 会话握手 | `{msg}` base64（原始 Noise 握手消息） |

### 8.2 剪贴板同步

发送方：

1. 本地剪贴板变化 → 构造 `Profile`（沿用现有 `text_hash` 计算）
2. `profile.source = 自身 did`
3. 发 `CLIPBOARD_SET {profile, seq}`

接收方：

1. `profile.source == 自身 did` → 丢弃（回环）
2. 调用 `ClipboardService::put(ctx, profile)`
   - `Saved` → 更新本地系统剪贴板，回 `CLIPBOARD_CHANGED`
   - `Unchanged` → 不做任何事（幂等已由现有实现保证）
   - `Err` → 回 `ERROR{code:6}`

### 8.3 冲突

语义为**最后写入胜出**，与现有 `put()` 覆盖行为一致。不做合并、不做向量时钟：剪贴板是单值资源，用户不会期望两端内容被拼接。

`seq` 仅用于同一连接内的补发序号，跨设备不可比。

## 9. 状态机

```
Idle
 ├─ start_pairing() ──> Advertising ──(对端连入)──> Handshaking ──(成功)──> Streaming
 │                         └─(60s 超时 / 3 次失败)──> Idle
 └─ connect(peer) ────> Connecting ──(TCP ok)──> NoiseHandshake ──(成功)──> Streaming

Streaming ──(空闲 30s / BYE / IO 错误)──> Idle
```

`Streaming` 下断线由上层按 §5.3 的降级顺序重连，**不保留任何需要恢复的协议状态**（幂等写入已保证重放安全）。

## 10. 错误码

| 码 | 名称 | 含义 |
|---|---|---|
| 0 | OK | 无错误 |
| 1 | VERSION_MISMATCH | 协议版本不兼容 |
| 2 | AUTH_FAILED | 码错误 / 确认失败 |
| 3 | NO_PAIRED_SESSION | 未配对却尝试会话握手 |
| 4 | BUSY | 已有配对或连接进行中 |
| 5 | PAYLOAD_TOO_LARGE | 帧超过 1 MiB |
| 6 | BAD_FRAME | 魔数 / 长度 / JSON 解析失败 |
| 7 | RATE_LIMITED | 触发频次限制 |
| 8 | INTERNAL | 内部错误 |

## 11. 测试向量

跨语言一致性是这类协议最易翻车的地方。向量已固化为文件，两侧单测**读同一份文件**：

| 文件 | 内容 |
|---|---|
| `libximep2p/crates/xime-p2p/tests/vectors/frames.json` | 帧编解码的正向用例 + 必须拒绝的帧头 |
| `libximep2p/crates/xime-p2p/tests/vectors/pairing.json` | 配对 KDF 派生结果、确认值、会话 prologue |

Android 侧的实现必须把这两份 JSON 当只读契约：改协议先改 JSON，两侧同时变红才算改对。

### 11.1 帧编解码

| 用例 | 输入 | 期望 hex |
|---|---|---|
| PING | type=0x0A, flags=0, payload=`{}` | `5850010A00000000027B7D` |
| ERROR | type=0x0C, flags=0, payload=`{"code":2}` | `5850010C000000000A7B22636F6465223A327D` |
| 空 payload | type=0x0B, flags=0, payload=`` | `5850010B0000000000` |
| 加密位 | type=0x08, flags=1, payload=`aabbcc` | `585001080100000003AABBCC` |
| 超限 | length=1048577 | 拒绝并断开 |

### 11.2 密码学

SPAKE2 与 Noise 的**实际输出依赖随机数，无法手工固化为固定值**，所以不编造这些向量。
可固化且真正决定互操作的是它们的确定性产物：

- 配对 KDF：给定 `K`、`nonce_a`、`nonce_b`，`k_c2s` / `k_s2c` / 确认值完全确定
- prologue：给定两个 `did`，字节序列完全确定

这两处一旦两侧不一致，表现是"配对总是失败"或"连上就掉"，且没有任何日志能指出原因，
因此必须用向量锁死。

至少覆盖：

- SPAKE2：固定码 + 固定随机数 → 固定 `K`
- Noise IK：固定密钥对 + 固定 nonce → 固定首包密文
- `text_hash`：与现有 `xime_sync_domain::hash::text_hash` 保持逐字节一致

## 12. 安全考量

| 威胁 | 对策 |
|---|---|
| 同网段窃听 | 全流量 AEAD，密钥不出端 |
| 离线暴力破解动态码 | PAKE 强化（SPAKE2），码不入网 |
| 在线暴力猜测 | 3 次失败作废 + 60 秒 TTL |
| 中间人 | PAKE 双向确认 + 长期公钥绑定 |
| 重放到其他设备对 | Noise prologue 绑定双方 `did` |
| 未认证连接耗尽资源 | 5 秒握手超时 + 并发上限 4 |
| 敏感内容外泄 | 见下 |

敏感内容处理（Android）：

- 读取 `ClipDescription.EXTRA_IS_SENSITIVE`（API 33+），命中则**不入同步队列**
- 长度超过 256 KiB 的文本默认不同步，可配置
- 不缓存明文到磁盘（`Profile` 仅存于内存与既有 store，不额外落盘）

## 13. 待定项

1. **crate 归属与 license**：建议 `xime-p2p`（纯 Rust）+ `xime-p2p-jni`（仅 Android 构建启用）。
   Android 绑定手写 JNI，风格与现有 `rime_jni.cc` / `onnx_jni.cc` 一致，不引入代码生成工具：
   对外仅 6 个函数、参数一律 JSON 字符串、事件用轮询而非回调，没有生成器的用武之地；
   同时避免在 F-Droid 可复现构建链条上多一个与工具版本相关的环节。
   license 与 libximecore 一致（GPL-3.0-or-later），协议核心已拆到独立仓 `libximep2p`。
2. **端口冲突策略**：固定端口顺延，还是每次随机后由 TXT 声明。倾向前者（防火墙规则稳定）。
3. **二维码扩展**：`xime://pair?v=1&did=..&code=..&addr=..` 可在动态码之外提供扫码路径，共享同一套握手。本版不实现。
4. **三端以上**：需要把 `seq` 换成按 `did` 分桶的向量时钟，届时 `ClipboardEvent` 需加 `origin` 字段。
5. **Android 侧保活**：短连接 + 补发已足够，是否需要前台服务待真机验证后再定。

## 14. 实现进度

> 协议核心已迁到独立仓 `libximep2p`（GPL-3.0-or-later）；下表「位置」与
> `cargo test -p xime-p2p` 都相对该仓。libximecore 经 git 依赖（pin rev）使用。

| 模块 | 状态 | 位置 |
|---|---|---|
| 帧编解码 | ✅ 已实现 | `src/frame.rs` |
| 设备标识与长期密钥 | ✅ 已实现 | `src/identity.rs` |
| 动态码（生成 / 分组 / 过期） | ✅ 已实现 | `src/code.rs` |
| SPAKE2 配对 | ✅ 已实现 | `src/pairing.rs` |
| Noise IK 会话 + 分片 | ✅ 已实现 | `src/session.rs` |
| 应用层 payload | ✅ 已实现 | `src/app.rs` |
| 已配对表持久化 | ✅ 已实现 | `src/store.rs` |
| 配对 / 会话握手驱动 | ✅ 已实现 | `src/proto.rs` |
| 跨语言测试向量 | ✅ 已固化 | `tests/vectors/*.json` |
| TCP 传输（监听 / 连接 / 端口顺延） | ✅ 已实现 | `src/transport.rs` |
| UDP 广播发现 | ✅ 已实现（受限广播） | `src/transport.rs` |
| 同步外观（Node：监听 / 重连 / 事件队列） | ✅ 已实现 | `src/node.rs` |
| 配对会话（阻塞封装 + 身份落盘） | ✅ 已实现 | `src/pair_session.rs` |
| mDNS 发现 | ⬜ 待实现 | 计划 `src/discovery/mdns.rs` |
| 定向广播（多网卡逐接口） | ⬜ 待实现 | 需引入网卡枚举依赖 |
| XimeYi（macOS）接入 | ✅ 已接入（同步 + 配对函数） | `clipboard_sync.rs` / `p2p_sync.rs` |
| 配对 UI（设置应用，`pair-page`） | ✅ 已实现 | `xime-setup/src/pages/pair.rs` |
| 传输方式选择器（关闭 / 插件 / 直连） | ✅ 已实现 | `pages/clipboard.rs` 的「剪贴板同步」标签页 |
| XimeYi 启用 `pair-page` | ⬜ 待 libximecore 推送后开启 | `ximeyi-setup/Cargo.toml` 的 features |
| Android JNI + 插件 | ⬜ 待实现 | 计划 `xime-p2p-jni` |

`cargo test -p xime-p2p` 共 75 个用例，覆盖帧向量、配对成功/失败、会话双向收发、
密文篡改、分片、配对表损坏，以及两条端到端路径：**两个 Node 真实 TCP 握手后双向
互发剪贴板**、**两端用动态码完成配对并各自写入配对表**。

`xime-p2p` 自身不含平台 API。`node` / `pair_session` 里用到 tokio，但对外只暴露
阻塞接口——宿主的同步 tick 与 Android JNI 都直接调，不需要它们依赖 tokio。

依赖统一在 digest 0.11 / aead 0.11 一代（`sha2 0.11` + `hmac 0.13` + `hkdf 0.13`
+ `chacha20poly1305 0.11` + `ed25519-dalek 3` + `x25519-dalek 3` + `snow 0.10`），
全部在根 `Cargo.toml` 的 `[workspace.dependencies]` 声明。
随机数用 `getrandom`，不引 `rand`——后者 0.9/0.10 两代之间 API 反复改名，对密码学库是没必要的不稳定源。

### 14.1 设置端配对界面

`xime-setup-lib` 的「设备关联」页（feature `pair-page`）是桌面端的配对入口：

- **数据目录**：`<用户数据根>/p2p`，即 `xime_config::get_data_dirs()` 里用户目录的
  父级。设置应用与输入法必须指向同一目录——各写各的，就成了「设置里配对成功、
  输入法仍认为未配对」，而界面上看不出任何异常。
- **线程模型**：显示端会话跑在后台 `std::thread`，结果经事件队列回传，UI 复用既有的
  250ms 轮询取走。界面层不引 async 运行时。
- **会话号**：重新生成码时递增。旧线程的收尾事件（往往晚到 1 秒）必须被丢弃，
  否则会把新会话的码清掉，表现为「点重新生成后一闪回到初始态」。
- **身份复用**：配对与应用同一份 `identity.bin`，did 由它派生，删除配对不会换身份。

已知冲突：输入端自动发现要绑 UDP 47822，而输入法常驻的 `Node` 也绑同一个端口。
同机「设置应用作为输入端」时发现会失败，界面提供「手动地址」兜底；跨机不受影响。
根治方案是给配对发现单独分配端口（协议 v0.3）。
