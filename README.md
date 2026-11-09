aj9│# rpi-adhd
aka│
akb│把 [i-have-adhd](https://github.com/ayghri/i-have-adhd) 的 ADHD 友好输出规则带进 rpi。
akc│
akd│规则本体只有一份：`src/adhd.md`（编译进二进制，无需外部文件）。每轮请求前追加到系统提示词，
ake│用 `/adhd` 开关。
akf│
akg│## 规则（10 条，全文见 `src/adhd.md`）
akh│
aki│1. 第一行就是下一步动作
akj│2. 多步任务编号
akk│3. 结尾给一个 2 分钟内能开始的动作
akl│4. 压住跑题
akm│5. 每轮复述状态
akn│6. 时间估算给具体单位
ako│7. 成果显式化
akp│8. 报错平铺直叙
akq│9. 列表每组最多 5 条
akr│10. 无开场白、无收尾客套、无复述
aks│
akt│## 安装
aku│
akv│在目标项目目录执行（路径改成本仓库实际位置）：
akw│
akx│```sh
aky│rpi package link --path /path/to/rpi-adhd
akz│```
akA│
akB│启动 `rpi` 后自动加载。验证：
akC│
akD│```sh
akE│printf '{"id":"1","type":"get_commands"}\n' | rpi --mode rpc --offline --approve | grep -o '"name":"adhd"'
akF│```
akG│
akH│卸载：`rpi package unlink rpi-adhd`。
akI│
akJ│## 用法

| 命令 | 行为 |
| --- | --- |
| `/adhd` | 切换开关并保存 |
| `/adhd on` / `off` | 直接设定并保存 |
| `/adhd status` | 显示当前状态与已保存的默认值 |

切换即写入配置，重启后保持。配置文件：`$RPI_AGENT_DIR/rpi-adhd.json`
（未设置该环境变量时用 `~/.rpi/agent/rpi-adhd.json`）：

```json
{ "enabled": true }
```

优先级：环境变量 `RPI_ADHD`（`1`/`on`/`true`，`0`/`off`/`false`）> 配置文件 > 默认 off。

## 限制

- 注入走 `register_before_agent_start`，旧宿主缺该槽位时插件加载失败（返回非零），不会静默无效。
- 配置是全局的（所有项目共用一个开关）；要按项目区分就改用 `RPI_ADHD` 环境变量。

## 开发
akZ│
ak0│```sh
ak1│cargo test && cargo clippy --all-targets && cargo fmt --check
ak2│rpi dev
ak3│```
ak4│
ak5│无网络访问，不写文件，不收集数据。MIT。