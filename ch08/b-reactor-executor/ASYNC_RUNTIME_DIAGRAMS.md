# 异步运行时系统架构图表


本文档包含了 Rust 异步运行时系统的详细架构图表，展示了 Task、子 Future、Executor、Reactor 等核心概念的关系和执行流程。

## 整体架构概览

```mermaid
graph TB
    subgraph "主线程 (Main Thread)"
        A["Executor 启动"] --> B["spawn Task"]
        B --> C["Task 调度循环"]
        C --> D["Task.poll 执行"]
        D --> E{"Task 状态?"}
        E -->|Ready| F["Task 完成"]
        E -->|NotReady| G["Task 重新存储"]
        F --> H{"还有 Task?"}
        G --> H
        H -->|是| I["主线程休眠 thread.park"]
        H -->|否| J["程序结束"]
        I --> K["等待唤醒"]
    end

    subgraph "Reactor 线程 (I/O Thread) - EventLoop"
        L["EventLoop 启动"] --> M["poll.poll 监听 I/O 事件"]
        M --> N{"I/O 事件就绪?"}
        N -->|否| M
        N -->|是| O["遍历 events.iter"]
        O --> P["提取 Token(id)"]
        P --> Q["查找 wakers.get(&id)"]
        Q --> R["调用 waker.wake"]
        R --> S["将 Task ID 加入就绪队列"]
        S --> T["唤醒主线程 thread.unpark"]
        T --> M
    end

    subgraph "Task 内部结构"
        U["Task = Box&lt;dyn Future&gt;"] --> V["状态机管理"]
        V --> W["子 Future 1"]
        V --> X["子 Future 2"]
        V --> Y["子 Future N"]
        
        W --> Z{"子 Future 类型?"}
        Z -->|I/O 型| AA["Reactor 管理"]
        Z -->|计算型| BB["父 Task 管理"]
        
        AA --> CC["Reactor ID"]
        BB --> DD["无独立 ID"]
    end

    subgraph "ID 系统"
        EE["Executor ID 系统"] --> FF["管理 Task 生命周期"]
        GG["Reactor ID 系统"] --> HH["管理 I/O 事件"]
        FF --> II["0, 1, 2, ..."]
        HH --> JJ["1, 2, 3, ..."]
    end

    subgraph "Waker 机制"
        KK["Waker 创建"] --> LL["包含 Task ID 和 Thread"]
        LL --> MM["wake 方法"]
        MM --> NN["将 Task ID 加入就绪队列"]
        MM --> OO["调用 thread.unpark"]
    end

    K -.->|"waker.wake"| T
    T --> C
    AA --> M
    CC --> P
    R --> KK

    style A fill:#e1f5fe
    style J fill:#c8e6c9
    style I fill:#fff3e0
    style L fill:#f3e5f5
    style M fill:#f3e5f5
    style R fill:#ffebee
    style U fill:#e8f5e8
    style AA fill:#fff8e1
    style BB fill:#f1f8e9
    style KK fill:#fff3e0
```

## Task 和子 Future 详细执行流程

```mermaid
sequenceDiagram
    participant Main as 主线程
    participant Exec as Executor
    participant Task as Task (Coroutine0)
    participant SubF1 as 子 Future 1 (HttpGetFuture)
    participant SubF2 as 子 Future 2 (HttpGetFuture)
    participant Reactor as Reactor 线程

    Main->>Exec: executor.block_on(async_main)
    Exec->>Exec: spawn(Coroutine0) - Task ID = 0
    Exec->>Task: poll(waker)
    
    Note over Task: State0::Start
    Task->>Task: 创建子 Future 1
    Task->>SubF1: poll(waker)
    
    Note over SubF1: 第一次 poll
    SubF1->>Reactor: register(stream, Interest::READABLE, Reactor ID = 1)
    SubF1->>Reactor: set_waker(waker, Reactor ID = 1)
    SubF1->>Task: PollState::NotReady (I/O 阻塞)
    Task->>Exec: PollState::NotReady
    Exec->>Exec: insert_task(0, Coroutine0)
    Exec->>Main: thread.park() - 主线程休眠

    Note over Reactor: I/O 事件监听
    Reactor->>Reactor: I/O 就绪事件
    Reactor->>Reactor: 查找 Waker (Reactor ID = 1)
    Reactor->>Exec: waker.wake() - 唤醒主线程
    Exec->>Main: thread.unpark()
    
    Main->>Exec: 继续执行
    Exec->>Task: poll(waker)
    Task->>SubF1: poll(waker)
    SubF1->>Task: PollState::Ready("HTTP 响应")
    
    Note over Task: State0::Wait1 -> State0::Wait2
    Task->>Task: 创建子 Future 2
    Task->>SubF2: poll(waker)
    
    Note over SubF2: 第一次 poll
    SubF2->>Reactor: register(stream, Interest::READABLE, Reactor ID = 2)
    SubF2->>Reactor: set_waker(waker, Reactor ID = 2)
    SubF2->>Task: PollState::NotReady (I/O 阻塞)
    Task->>Exec: PollState::NotReady
    Exec->>Exec: insert_task(0, Coroutine0)
    Exec->>Main: thread.park() - 主线程休眠

    Note over Reactor: 第二个 I/O 事件
    Reactor->>Reactor: I/O 就绪事件
    Reactor->>Reactor: 查找 Waker (Reactor ID = 2)
    Reactor->>Exec: waker.wake() - 唤醒主线程
    Exec->>Main: thread.unpark()
    
    Main->>Exec: 继续执行
    Exec->>Task: poll(waker)
    Task->>SubF2: poll(waker)
    SubF2->>Task: PollState::Ready("HTTP 响应")
    
    Note over Task: State0::Wait2 -> State0::Resolved
    Task->>Exec: PollState::Ready("")
    Exec->>Main: 所有任务完成
```

## 数据结构和关系图

```mermaid
graph TB
    subgraph "Executor 管理"
        A[ExecutorCore] --> B["tasks: HashMap&lt;usize, Task&gt;"]
        A --> C["ready_queue: Vec&lt;usize&gt;"]
        A --> D["next_id: Cell&lt;usize&gt;"]
        
        B --> E["Task ID = 0: Box&lt;Coroutine0&gt;"]
        C --> F["就绪队列: [0]"]
        D --> G["ID 计数器: 0, 1, 2, ..."]
    end

    subgraph "Reactor 管理"
        H[Reactor] --> I["wakers: HashMap&lt;usize, Waker&gt;"]
        H --> J["next_id: AtomicUsize"]
        
        I --> K["Reactor ID = 1: Waker"]
        I --> L["Reactor ID = 2: Waker"]
        J --> M["ID 计数器: 1, 2, 3, ..."]
    end

    subgraph "Task 内部结构"
        E --> N[Coroutine0]
        N --> O["state: State0"]
        O --> P["Wait1: HttpGetFuture"]
        O --> Q["Wait2: HttpGetFuture"]
        
        P --> R["Reactor ID = 1"]
        Q --> S["Reactor ID = 2"]
    end

    subgraph "Waker 连接"
        K --> T["Waker {id: 0, thread: MainThread}"]
        L --> U["Waker {id: 0, thread: MainThread}"]
        T --> V["唤醒 Task ID = 0"]
        U --> V
    end

    R -.->|"I/O 事件"| K
    S -.->|"I/O 事件"| L
    V --> C

    style A fill:#e1f5fe
    style H fill:#f3e5f5
    style N fill:#e8f5e8
    style T fill:#fff8e1
    style U fill:#fff8e1
```

## 子 Future 管理方式对比

```mermaid
graph TD
    subgraph "I/O 密集型子 Future"
        A[HttpGetFuture] --> B[有 Reactor ID]
        A --> C[注册到 Reactor]
        A --> D[设置 Waker 映射]
        A --> E[I/O 阻塞时返回 NotReady]
        A --> F[通过 Reactor 唤醒]
    end

    subgraph "计算密集型子 Future"
        G[JoinAll] --> H[无独立 ID]
        G --> I[不注册到 Reactor]
        G --> J[直接调用子 Future.poll]
        G --> K[通过父 Task 管理]
        G --> L[不涉及 I/O 操作]
    end

    subgraph "管理方式"
        M[Reactor 管理] --> N[I/O 事件监听]
        M --> O[Waker 唤醒机制]
        M --> P[跨线程通信]
        
        Q[父 Task 管理] --> R[状态机控制]
        Q --> S[直接方法调用]
        Q --> T[单线程执行]
    end

    A --> M
    G --> Q

    style A fill:#fff8e1
    style G fill:#f1f8e9
    style M fill:#f3e5f5
    style Q fill:#e8f5e8
```

## 核心概念总结

### Task 和 Future 的关系

- **Task** = `Box<dyn Future<Output = String>>`（类型别名）
- **Task** 强调**可调度性**：被 Executor 管理的执行单元
- **Future** 强调**异步性**：异步计算的抽象
- **一个 Task 可以包含多个子 Future**

### 层次结构

```
Task (Coroutine0) - Executor ID = 0
├── 子 Future 1 (HttpGetFuture) - Reactor ID = 1
│   └── 执行: HTTP 请求 "/600/HelloAsyncAwait"
└── 子 Future 2 (HttpGetFuture) - Reactor ID = 2
    └── 执行: HTTP 请求 "/400/HelloAsyncAwait"
```

### 管理方式

**Task 管理**：
- 被 Executor 直接管理和调度
- 有唯一的 Executor ID
- 存储在 `tasks` HashMap 中
- 可以被暂停、恢复、完成

**子 Future 管理**：
- 被父 Task 内部管理
- 通过状态机控制执行顺序
- I/O 型有自己的 Reactor ID（用于 I/O 事件）
- 计算型无独立 ID，通过父 Task 直接管理

### ID 系统设计

**两套独立的 ID 系统**：

1. **Executor ID 系统**：
   - 管理所有 Future 任务的生命周期
   - 用于任务调度和存储
   - 在单线程环境中使用（线程安全）

2. **Reactor ID 系统**：
   - 管理 I/O 事件和 Waker 的映射
   - 用于跨线程通信（Reactor 在独立线程中运行）
   - 需要线程安全的 ID 生成

### 子 Future 分类

**I/O 密集型子 Future**：
- 通过 Reactor 管理
- 有 Reactor ID
- 需要异步 I/O 操作
- 例子：`HttpGetFuture`、文件操作 Future

**计算密集型子 Future**：
- 通过父 Task 直接管理
- 无独立 ID
- 纯计算，不涉及 I/O
- 例子：`JoinAll`、数学计算 Future

### 设计优势

**单 Task + 多子 Future 模式**：
- **顺序执行**：子 Future 按状态机顺序执行
- **状态管理**：通过状态机管理多个子 Future
- **资源效率**：只需要管理一个 Task
- **简单性**：避免复杂的多 Task 调度

这种设计实现了**单任务 + 状态机**的协程模式，既保持了简单性，又支持了复杂的异步操作组合。
