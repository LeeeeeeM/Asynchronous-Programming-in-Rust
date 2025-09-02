# 高级架构 Demo 使用指南

## 概述

这个高级架构demo展示了如何实现**多Executor + 多Reactor**的异步运行时系统，相比原始的单一架构，具有更好的扩展性和灵活性。

## 架构特性

### 1. 多Executor架构
- **网络执行器**: 专门处理网络I/O任务
- **文件执行器**: 专门处理文件系统任务  
- **定时器执行器**: 专门处理定时器任务

### 2. 多Reactor架构
- **网络Reactor**: 监听网络I/O事件
- **文件Reactor**: 监听文件系统事件
- **定时器Reactor**: 监听定时器事件

### 3. 解耦设计
- Executor和Reactor通过Waker间接通信
- 每个Reactor知道应该唤醒哪个Executor
- 支持动态注册和注销

## 文件结构

```
src/
├── advanced_runtime.rs    # 高级运行时实现
├── advanced_main.rs       # 高级架构演示主程序
├── main.rs               # 原始架构演示
└── runtime/              # 原始运行时实现
    ├── executor.rs
    └── reactor.rs
```

## 运行方式

### 运行原始架构演示
```bash
cargo run --bin app
```

### 运行高级架构演示
```bash
cargo run --bin advanced
```

## 核心组件详解

### ExecutorRegistry (执行器注册表)
```rust
pub struct ExecutorRegistry {
    executors: Arc<Mutex<HashMap<ExecutorId, Arc<dyn TaskExecutor>>>>,
}
```

**功能**:
- 管理多个Executor实例
- 提供Executor的注册、查找、列表功能
- 支持动态添加和移除Executor

### ReactorManager (Reactor管理器)
```rust
pub struct ReactorManager {
    reactors: Arc<Mutex<HashMap<ReactorType, Arc<dyn Reactor>>>>,
}
```

**功能**:
- 管理不同类型的Reactor
- 统一启动和停止所有Reactor
- 支持Reactor的动态注册

### Waker (任务唤醒器)
```rust
pub struct Waker {
    task_id: TaskId,
    executor_id: ExecutorId,  // 关键：指定唤醒哪个Executor
}
```

**改进点**:
- 不再绑定到特定线程
- 通过executor_id指定唤醒目标
- 支持跨线程唤醒

## 执行流程

### 1. 初始化阶段
```
1. 创建ExecutorRegistry和ReactorManager
2. 注册不同类型的Reactor
3. 创建并注册多个Executor
4. 启动所有Reactor
```

### 2. 任务执行阶段
```
1. 创建不同类型的任务
2. 将任务分配给对应的Executor
3. 每个Executor在独立线程中运行
4. 任务注册到对应的Reactor
```

### 3. 事件处理阶段
```
1. Reactor监听对应类型的事件
2. 事件发生时，通过Waker唤醒对应的Executor
3. Executor处理被唤醒的任务
4. 任务完成后从Executor中移除
```

## 演示输出示例

```
=== 多Executor + 多Reactor 架构演示 ===

注册执行器: 网络执行器 (ID: 1)
注册执行器: 文件执行器 (ID: 2)
注册执行器: 定时器执行器 (ID: 3)

已注册的执行器:
  - 网络执行器 (ID: 1)
  - 文件执行器 (ID: 2)
  - 定时器执行器 (ID: 3)

已注册的Reactor:
  - network
  - file
  - timer

创建任务...

开始执行任务...

网络执行器: 开始运行
文件执行器: 开始运行
定时器执行器: 开始运行

网络Reactor: 注册任务 101 的waker
网络Reactor: 注册任务 102 的waker
文件Reactor: 注册任务 201 的waker
文件Reactor: 注册任务 202 的waker
定时器Reactor: 注册任务 301 的waker
定时器Reactor: 注册任务 302 的waker

网络执行器: 还有 2 个待处理任务，等待唤醒...
文件执行器: 还有 2 个待处理任务，等待唤醒...
定时器执行器: 还有 2 个待处理任务，等待唤醒...

网络Reactor: 触发网络事件，唤醒任务 101
网络Reactor: 触发网络事件，唤醒任务 102
Waker: 唤醒任务 101 (执行器 1)
Waker: 唤醒任务 102 (执行器 1)
网络执行器: 唤醒任务 101
网络执行器: 唤醒任务 102

网络执行器: 任务 101 完成，结果: 网络任务 101 已完成
网络执行器: 任务 102 完成，结果: 网络任务 102 已完成
网络执行器: 所有任务完成

文件Reactor: 触发文件事件，唤醒任务 201
文件Reactor: 触发文件事件，唤醒任务 202
Waker: 唤醒任务 201 (执行器 2)
Waker: 唤醒任务 202 (执行器 2)
文件执行器: 唤醒任务 201
文件执行器: 唤醒任务 202

文件执行器: 任务 201 完成，结果: 文件任务 201 已完成
文件执行器: 任务 202 完成，结果: 文件任务 202 已完成
文件执行器: 所有任务完成

定时器Reactor: 触发定时器事件，唤醒任务 301
定时器Reactor: 触发定时器事件，唤醒任务 302
Waker: 唤醒任务 301 (执行器 3)
Waker: 唤醒任务 302 (执行器 3)
定时器执行器: 唤醒任务 301
定时器执行器: 唤醒任务 302

定时器执行器: 任务 301 完成，结果: 定时器任务 301 已完成
定时器执行器: 任务 302 完成，结果: 定时器任务 302 已完成
定时器执行器: 所有任务完成

=== 演示完成 ===
```

## 架构优势

### 1. 真正的多线程执行
- 每个Executor在独立线程中运行
- 可以充分利用多核CPU
- 不同类型的任务可以并行处理

### 2. 专业化处理
- 网络任务由网络Executor处理
- 文件任务由文件Executor处理
- 定时器任务由定时器Executor处理

### 3. 精确唤醒
- 网络事件只唤醒网络Executor
- 文件事件只唤醒文件Executor
- 避免了不必要的唤醒

### 4. 资源隔离
- 网络I/O不会阻塞文件I/O
- 文件I/O不会阻塞定时器I/O
- 提高了系统的响应性

### 5. 动态扩展
- 可以动态添加新的Executor
- 可以动态添加新的Reactor类型
- 支持运行时配置

## 与原始架构的对比

| 特性 | 原始架构 | 高级架构 |
|------|----------|----------|
| 执行模型 | 单线程执行 | 多线程执行 |
| Reactor数量 | 单一Reactor | 多个Reactor |
| 任务调度 | 全局队列 | 分类型调度 |
| 扩展性 | 有限 | 高度可扩展 |
| 资源利用 | 单核 | 多核 |
| 复杂度 | 简单 | 中等 |

## 实际应用场景

### 1. Web服务器
- 网络Executor处理HTTP请求
- 文件Executor处理静态文件
- 定时器Executor处理会话超时

### 2. 数据库系统
- 网络Executor处理客户端连接
- 文件Executor处理日志写入
- 定时器Executor处理连接池管理

### 3. 游戏服务器
- 网络Executor处理玩家通信
- 文件Executor处理存档操作
- 定时器Executor处理游戏逻辑

## 总结

这个高级架构demo展示了现代异步运行时系统的设计思想，通过解耦Executor和Reactor，实现了真正的多线程执行和专业化I/O处理。相比原始的单一架构，具有更好的性能、扩展性和灵活性，是学习Tokio等现代异步运行时内部工作机制的绝佳示例。
