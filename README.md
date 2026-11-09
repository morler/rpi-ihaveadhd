# rpi-adhd

把 [i-have-adhd](https://github.com/ayghri/i-have-adhd) 的 ADHD 友好输出规则带进 [rpi](https://github.com/earendil-works/rpi)：让 agent 的回复第一行就是下一步动作、多步任务编号、结尾给一个 2 分钟内能开始的动作。

规则本体只有一份：[`src/adhd.md`](src/adhd.md)，编译进二进制（`include_str!`），不需要外部文件。开启时在每次请求前追加到系统提示词，用 `/adhd` 开关。

## 安装

要求 rpi 0.21+。预编译包从 GitHub Release 下载（不经 crates.io，所以必须带上 `--github`）：

```sh
rpi install rpi-adhd --github morler/rpi-adhd --version 0.1.0
```

支持 `x86_64-unknown-linux-gnu`、`aarch64-unknown-linux-gnu`、`x86_64-pc-windows-msvc`、
`x86_64-apple-darwin`、`aarch64-apple-darwin`。musl 平台不支持，原因见[限制](#限制)。

从源码装（需要 Rust 工具链）：

```sh
git clone https://github.com/morler/rpi-adhd.git
cd /path/to/your-project
rpi package link --path /path/to/rpi-adhd
```

启动 `rpi` 后自动加载。验证命令已经注册：

```sh
printf '{"id":"1","type":"get_commands"}\n' | rpi --mode rpc --offline --approve | grep -o '"name":"adhd"'
```

卸载：`rpi package unlink rpi-adhd`。

## 用法

| 命令 | 行为 |
| --- | --- |
| `/adhd` | 切换开关并保存 |
| `/adhd on` / `/adhd off` | 直接设定并保存（`start` / `stop` 同义） |
| `/adhd status` | 显示当前状态与已保存的默认值 |

切换即写入配置，重启后保持。配置文件：`$RPI_AGENT_DIR/rpi-adhd.json`
（未设置该环境变量时用 `~/.rpi/agent/rpi-adhd.json`）：

```json
{ "enabled": true }
```

写入是原子的（临时文件 + rename），文件里其它键会保留。

### 优先级

1. 环境变量 `RPI_ADHD`：`1`/`on`/`true` 或 `0`/`off`/`false`
2. 配置文件 `enabled`
3. 默认 off

环境变量在两者冲突时决定启动默认值，`/adhd status` 会如实显示。

## 规则

全文见 [`src/adhd.md`](src/adhd.md)，共 10 条：

1. 第一行就是下一步动作
2. 多步任务编号
3. 结尾给一个 2 分钟内能开始的动作
4. 压住跑题
5. 每轮复述状态
6. 时间估算给具体单位
7. 成果显式化
8. 报错平铺直叙
9. 列表每组最多 5 条
10. 无开场白、无收尾客套、无复述

## 限制

- 注入走 `register_before_agent_start`，旧宿主缺该槽位时插件加载失败（返回非零），不会静默无效。
- 配置是全局的（所有项目共用一个开关）；要按项目区分就改用 `RPI_ADHD` 环境变量。
- 只影响系统提示词，不拦截或改写模型输出。
- 没有 musl 构建：musl target 不产出 cdylib（`dropping unsupported crate type`），且静态链接的 musl 宿主没有
  `dlopen`（`Dynamic loading not supported`），插件机制本身就用不了。

## 隐私

无网络访问，不写文件（除 `/adhd` 主动写入的配置文件），不收集数据。

## 开发

```sh
cargo test && cargo clippy --all-targets && cargo fmt --check
rpi dev
```

## 许可证

MIT，见 [LICENSE](LICENSE)。
