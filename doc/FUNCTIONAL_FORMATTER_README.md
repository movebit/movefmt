# 函数式格式化器集成说明

## 概述

我们已经成功将重构后的函数式格式化器集成到了 movefmt 工程中。新的格式化器采用不可变状态设计，替代了原有的基于 `RefCell`/`Cell` 的可变状态管理。

## 文件结构

```
src/
├── core/
│   ├── fmt.rs          # 原有的格式化器
│   ├── fmt_state.rs    # 新的函数式格式化器
│   ├── mod.rs          # 更新了模块导出
│   └── token_tree.rs
├── bin/
│   └── main.rs         # 更新了命令行集成
└── ...
```

## 主要改进

### 1. 状态管理优化
- **之前**: 使用 `RefCell<String>`, `Cell<u32>` 等可变状态
- **现在**: 使用不可变的 `FormatState` 结构体，通过函数式编程传递状态

### 2. 性能提升
- **消除运行时借用检查** - 不再需要 `RefCell` 的运行时开销
- **更好的内存局部性** - 状态数据紧密排列
- **编译时优化** - 编译器可以更好地优化不可变操作
- **线程安全** - 天然支持并发处理

### 3. 安全性提升
- 消除了多重借用导致的 panic 风险
- 天然的线程安全性
- 更清晰的数据流

## 使用方法

### 命令行使用

```bash
# 使用原有的格式化器（默认）
./movefmt test_functional_formatter.move

# 使用新的函数式格式化器（实验性）
./movefmt --functional test_functional_formatter.move

# 查看详细输出
./movefmt --functional --verbose test_functional_formatter.move

# 从标准输入使用函数式格式化器
echo "fun test(){return 42;}" | ./movefmt --functional --stdin
```


## 核心组件

### FormatState
不可变的格式化状态，包含：
- `output`: 当前输出内容
- `cur_line`: 当前行号
- `depth`: 缩进深度
- `comments_index`: 注释处理索引
- `pre_simple_token`: 前一个token
- `cur_nested_kind`: 当前嵌套类型

### FunctionalFormat
主要的格式化器类，提供：
- `new()`: 创建格式化器实例
- `format_token_trees()`: 执行格式化
- 各种状态转换方法

## 测试

```bash
# 运行单元测试
cargo test fmt_state

# 测试基本功能
cargo run --bin movefmt -- --functional test_functional_formatter.move

# 对比新旧格式化器输出
cargo run --bin movefmt -- test_functional_formatter.move > old_output.move
cargo run --bin movefmt -- --functional test_functional_formatter.move > new_output.move
diff old_output.move new_output.move
```


## 性能基准

```bash
# 运行性能测试（需要实现）
cargo bench --bench format_benchmark

# 内存使用分析
valgrind --tool=massif cargo run --bin movefmt -- --functional large_file.move
```

这个集成为 movefmt 提供了一个更现代、更安全、更高性能的格式化器选项，同时保持了向后兼容性。

## 当前状态

### 已完成
- ✅ 基本的状态管理重构
- ✅ 增加命令行选项`--functional`
- ✅ 基本的格式化流程, 原有功能保持不变, 向后兼容
- ✅ 测试框架
- ✅ 支持现有的配置文件

### 待完成（需要进一步开发）
- ⏳ 完整的嵌套token处理逻辑
- ⏳ 所有分支处理逻辑的迁移
- ⏳ 注释处理的完整实现
- ⏳ 性能优化和并发支持
- ⏳ 全面的测试覆盖

## 迁移策略

1. **第一阶段** ✅ - 基础框架, 命令行集成
2. **第二阶段** - 逐步迁移复杂的格式化逻辑, 命令行开放
3. **第三阶段** - 性能优化和并发支持
4. **第四阶段** - 完全替换原有实现

## 注意事项

1. **实验性功能**: 当前的函数式格式化器还是实验性的，某些复杂的格式化场景可能还未完全实现。

2. **向后兼容**: 原有的格式化器仍然是默认选项，确保现有用户不受影响。

3. **性能**: 虽然理论上新的实现应该更快，但在完全优化之前可能存在性能差异。

4. **错误处理**: 新的实现使用了更好的错误处理机制，但可能与原有的错误信息格式不完全一致。

## 贡献指南

如果你想帮助完善函数式格式化器：

1. 查看 `src/core/fmt_state.rs` 中标记为 "简化实现" 的方法
2. 参考 `src/core/fmt.rs` 中的原始实现
3. 逐步迁移复杂的逻辑到函数式版本
4. 添加相应的测试用例
5. 提交 PR 并说明改进内容
