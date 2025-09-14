# Proto-Sign

Protocol Buffers 兼容性检查和语义指纹工具。

## 功能特性

- **破坏性变更检测**: 全面的基于规则的 protobuf 兼容性检查
- **语义指纹生成**: 生成稳定的指纹，忽略格式变更
- **Buf 兼容**: 与 Buf 相同的规则和行为
- **灵活配置**: 基于 YAML 的配置，提供预设模板

## 安装

```bash
# 从源码构建
git clone https://github.com/your-org/proto-sign.git
cd proto-sign
cargo install --path .

# 提取测试配置（开发时需要）
bash ./compat-configs/extract_buf_configs.sh
```

## 使用方法

### 破坏性变更检测

```bash
# 基本的破坏性变更检查
proto-sign breaking old.proto new.proto

# JSON 输出
proto-sign breaking old.proto new.proto --format json

# 使用特定规则分类
proto-sign breaking old.proto new.proto --use-categories FILE,WIRE
```

### 快速兼容性检查

```bash
# 三级兼容性评估 (绿色/黄色/红色)
proto-sign compare old.proto new.proto
```

### 语义指纹生成

```bash
# 生成语义指纹
proto-sign fingerprint file.proto
```

## 配置

Proto-Sign 使用 YAML 配置文件。复制模板开始使用：

```bash
# 选择配置模板
cp compat-configs/examples/strict-mode.yaml proto-sign.yaml

# 使用自定义配置
proto-sign breaking old.proto new.proto --config proto-sign.yaml
```

### 配置模板

- **`strict-mode.yaml`** - 所有规则分类（推荐用于公共 API）
- **`lenient-mode.yaml`** - 内部 API 的平衡模式
- **`wire-only.yaml`** - 仅二进制兼容性
- **`specific-rules.yaml`** - 自定义规则选择

### 配置格式

```yaml
version: v1
breaking:
  use_categories:
    - FILE
    - PACKAGE
    - WIRE
    - WIRE_JSON
  except_rules:
    - FIELD_SAME_JSON_NAME
  ignore:
    - "generated/**"
  ignore_unstable_packages: true
```

## 规则分类

- **FILE** - 文件级变更（删除、包变更）
- **PACKAGE** - 包级变更（消息/服务删除）
- **WIRE** - 二进制编码兼容性
- **WIRE_JSON** - JSON 序列化兼容性

## 库使用

```rust
use proto_sign::spec::{Spec, Compatibility};

let old_spec = Spec::try_from(old_proto_content)?;
let new_spec = Spec::try_from(new_proto_content)?;

match old_spec.compare_with(&new_spec) {
    Compatibility::Green => println!("无变更"),
    Compatibility::Yellow => println!("向后兼容"),
    Compatibility::Red => println!("检测到破坏性变更"),
}

// 详细分析
let result = old_spec.check_breaking_changes(&new_spec);
for change in result.changes {
    println!("{}: {}", change.rule_id, change.message);
}
```

## 兼容性等级

- **绿色**: 文件在语义上完全相同
- **黄色**: 新文件向后兼容旧文件
- **红色**: 存在破坏性变更

## 许可证

Apache License 2.0

## 致谢

破坏性变更检测规则移植自 [Buf](https://github.com/bufbuild/buf)。