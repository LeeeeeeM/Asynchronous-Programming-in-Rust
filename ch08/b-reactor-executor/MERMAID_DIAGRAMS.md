# b-reactor-executor 项目 Mermaid 流程图

## 1. 原始单线程架构流程图

```mermaid
graph TD
    A[主程序启动] --> B[调用 runtime::init]
    B --> C[启动 Reactor 线程]
    B --> D[创建 Executor]
    C --> E[Reactor 开始监听 I/O 事件]
    D --> F[调用 executor.block_on]
    
    F --> G[spawn 初始任务到就绪队列]
    G --> H[开始主循环]
    
    H --> I{就绪队列有任务?}
    I -->|是| J[取出任务ID]
    J --> K[获取 Future 任务]
    K --> L[创建 Waker]
    L --> M[调用 future.poll]
    
    M --> N{任务状态?}
    N -->|Ready| O[任务完成，继续下一个]
    N -->|NotReady| P[将任务重新插入任务列表]
    
    O --> I
    P --> I
    
    I -->|否| Q{还有未完成任务?}
    Q -->|是| R[主线程休眠 thread.park]
    Q -->|否| S[所有任务完成，退出]
    
    R --> T[Reactor Event Loop 开始]
    T --> U{I/O 事件发生?}
    U -->|否| T
    U -->|是| V[获取对应的 Waker]
    V --> W[调用 waker.wake]
    
    W --> X[将任务ID放入就绪队列]
    X --> Y[唤醒主线程 thread.unpark]
    Y --> H
    
    subgraph "Reactor 线程 - Event Loop"
        E
        T
        U
        V
        W
        X
        Y
    end
    
    subgraph "主线程"
        A
        B
        D
        F
        G
        H
        I
        J
        K
        L
        M
        N
        O
        P
        Q
        R
        S
    end
    
    style A fill:#e1f5fe
    style S fill:#c8e6c9
    style R fill:#fff3e0
    style E fill:#f3e5f5
    style W fill:#ffebee
```

## 2. 高级多线程架构流程图

```mermaid
graph TD
    A[主程序启动] --> B[初始化 ExecutorRegistry]
    B --> C[初始化 ReactorManager]
    C --> D[注册多个 Reactor]
    D --> E[注册多个 Executor]
    E --> F[启动所有 Reactor]
    F --> G[创建不同类型的任务]
    G --> H[将任务分配给对应 Executor]
    H --> I[在独立线程中运行每个 Executor]
    
    subgraph "网络 Executor 线程"
        J1[网络 Executor 运行]
        J2[处理网络任务]
        J3[创建网络 Waker]
        J4[调用 future.poll]
        J5[注册到网络 Reactor]
        J6[等待网络事件]
    end
    
    subgraph "文件 Executor 线程"
        K1[文件 Executor 运行]
        K2[处理文件任务]
        K3[创建文件 Waker]
        K4[调用 future.poll]
        K5[注册到文件 Reactor]
        K6[等待文件事件]
    end
    
    subgraph "定时器 Executor 线程"
        L1[定时器 Executor 运行]
        L2[处理定时器任务]
        L3[创建定时器 Waker]
        L4[调用 future.poll]
        L5[注册到定时器 Reactor]
        L6[等待定时器事件]
    end
    
    subgraph "网络 Reactor 线程 - Event Loop"
        M1
        M2
        M3
        M4
        M5
    end
    
    subgraph "文件 Reactor 线程 - Event Loop"
        N1
        N2
        N3
        N4
        N5
    end
    
    subgraph "定时器 Reactor 线程 - Event Loop"
        O1
        O2
        O3
        O4
        O5
    end
    
    I --> J1
    I --> K1
    I --> L1
    
    J1 --> J2
    J2 --> J3
    J3 --> J4
    J4 --> J5
    J5 --> J6
    
    K1 --> K2
    K2 --> K3
    K3 --> K4
    K4 --> K5
    K5 --> K6
    
    L1 --> L2
    L2 --> L3
    L3 --> L4
    L4 --> L5
    L5 --> L6
    
    J5 --> M1
    K5 --> N1
    L5 --> O1
    
    M1 --> M2
    M2 --> M3
    M3 --> M4
    M4 --> M5
    M5 --> J2
    
    N1 --> N2
    N2 --> N3
    N3 --> N4
    N4 --> N5
    N5 --> K2
    
    O1 --> O2
    O2 --> O3
    O3 --> O4
    O4 --> O5
    O5 --> L2
    
    style A fill:#e1f5fe
    style J1 fill:#e8f5e8
    style K1 fill:#fff3e0
    style L1 fill:#f3e5f5
    style M1 fill:#ffebee
    style N1 fill:#e0f2f1
    style O1 fill:#fce4ec
```

## 3. 核心组件交互图

```mermaid
graph LR
    subgraph "Future 层"
        F1
        F2
        F3
        F4
    end
    
    subgraph "Executor 层"
        E1
        E2
        E3
        E4
    end
    
    subgraph "Reactor 层"
        R1
        R2
        R3
        R4
    end
    
    subgraph "Waker 机制"
        W1
        W2
        W3
    end
    
    F1 --> E1
    F2 --> E2
    F3 --> E3
    F4 --> E4
    
    E1 --> R1
    E2 --> R2
    E3 --> R3
    E4 --> R4
    
    E1 --> W1
    E2 --> W1
    E3 --> W1
    E4 --> W1
    
    W1 --> W2
    W2 --> R1
    W2 --> R2
    W2 --> R3
    W2 --> R4
    
    R1 --> W3
    R2 --> W3
    R3 --> W3
    R4 --> W3
    
    W3 --> E1
    W3 --> E2
    W3 --> E3
    W3 --> E4
    
    style F1 fill:#e1f5fe
    style E1 fill:#e8f5e8
    style R1 fill:#fff3e0
    style W1 fill:#f3e5f5
```

## 4. 数据流图

```mermaid
graph TD
    A[用户代码] --> B[async_main 函数]
    B --> C[HttpGetFuture 创建]
    C --> D[Executor.spawn]
    D --> E[任务进入就绪队列]
    
    E --> F[Executor 主循环]
    F --> G[取出任务]
    G --> H[创建 Waker]
    H --> I[调用 future.poll]
    
    I --> J{任务状态}
    J -->|Ready| K[任务完成]
    J -->|NotReady| L[注册到 Reactor]
    
    L --> M[Reactor 监听 I/O]
    M --> N[I/O 事件发生]
    N --> O[查找对应 Waker]
    O --> P[调用 waker.wake]
    
    P --> Q[任务ID 进入就绪队列]
    Q --> R[唤醒 Executor 线程]
    R --> F
    
    K --> S[继续下一个任务]
    S --> F
    
    subgraph "数据存储"
        T1
        T2
        T3
    end
    
    E --> T1
    G --> T2
    L --> T3
    Q --> T1
    O --> T3
    
    style A fill:#e1f5fe
    style K fill:#c8e6c9
    style P fill:#ffebee
    style T1 fill:#f0f0f0
    style T2 fill:#f0f0f0
    style T3 fill:#f0f0f0
```

## 5. 时序图

```mermaid
sequenceDiagram
    participant Main as 主线程
    participant Exec as Executor
    participant Future as HttpGetFuture
    participant Reactor as Reactor
    participant IO as I/O 系统
    
    Main->>Exec: executor.block_on(async_main)
    Exec->>Future: spawn(HttpGetFuture)
    Future->>Exec: 任务进入就绪队列
    
    loop 主循环
        Exec->>Future: future.poll(waker)
        Future->>Reactor: register(stream, waker)
        Future->>Exec: PollState::NotReady
        Exec->>Main: thread.park() 休眠
        
        Reactor->>IO: poll() 监听 I/O 事件
        IO-->>Reactor: I/O 事件就绪
        Reactor->>Reactor: 查找对应 waker
        Reactor->>Exec: waker.wake()
        Exec->>Main: thread.unpark() 唤醒
        
        Main->>Exec: 继续执行
        Exec->>Future: future.poll(waker)
        Future->>Exec: PollState::Ready(result)
        Exec->>Main: 任务完成
    end
```

## 6. 系统架构概览图

```mermaid
graph TB
    subgraph "应用层"
        APP[用户应用程序]
        ASYNC[async/await 代码]
    end
    
    subgraph "运行时层"
        subgraph "原始架构"
            EXEC1[单线程 Executor]
            REACT1[单 Reactor]
        end
        
        subgraph "高级架构"
            REG[ExecutorRegistry]
            RM[ReactorManager]
            EXEC2[多线程 Executor]
            REACT2[多 Reactor]
        end
    end
    
    subgraph "系统层"
        MIO[mio 库]
        THREAD[线程管理]
        IO[I/O 系统]
    end
    
    APP --> ASYNC
    ASYNC --> EXEC1
    ASYNC --> EXEC2
    
    EXEC1 --> REACT1
    EXEC2 --> REG
    REG --> EXEC2
    EXEC2 --> REACT2
    REACT2 --> RM
    
    REACT1 --> MIO
    REACT2 --> MIO
    MIO --> IO
    
    EXEC1 --> THREAD
    EXEC2 --> THREAD
    
    style APP fill:#e1f5fe
    style EXEC1 fill:#e8f5e8
    style EXEC2 fill:#e8f5e8
    style REACT1 fill:#fff3e0
    style REACT2 fill:#fff3e0
    style MIO fill:#f3e5f5
```

## 7. Waker 生命周期图

```mermaid
stateDiagram-v2
    [*] --> 创建: Executor 创建 Waker
    创建 --> 注册: 注册到 Reactor
    注册 --> 等待: 等待 I/O 事件
    等待 --> 唤醒: I/O 事件发生
    唤醒 --> 执行: 任务重新执行
    执行 --> 完成: 任务完成
    完成 --> [*]
    
    等待 --> 重新注册: 任务重新轮询
    重新注册 --> 等待
    
    唤醒 --> 等待: 任务仍未就绪
    
    note right of 创建
        Waker 包含:
        - task_id
        - executor_id
        - thread 引用
    end note
    
    note right of 注册
        Reactor 存储:
        - task_id -> Waker 映射
    end note
    
    note right of 唤醒
        waker.wake():
        1. 将 task_id 加入就绪队列
        2. 唤醒 Executor 线程
    end note
```

## 8. Event Loop 详细流程图

```mermaid
graph TD
    A[Reactor 启动] --> B[创建 Poll 实例]
    B --> C[创建 Events 缓冲区]
    C --> D[开始 Event Loop]
    
    D --> E[poll.poll 监听 I/O 事件]
    E --> F{有事件就绪?}
    F -->|否| G[继续监听]
    G --> E
    
    F -->|是| H[遍历就绪的事件]
    H --> I[获取事件 Token]
    I --> J[从 Token 提取任务 ID]
    J --> K[查找对应的 Waker]
    K --> L{Waker 存在?}
    
    L -->|否| M[跳过此事件]
    L -->|是| N[调用 waker.wake]
    
    M --> O{还有更多事件?}
    N --> P[将任务 ID 加入就绪队列]
    P --> Q[唤醒 Executor 线程]
    Q --> O
    
    O -->|是| H
    O -->|否| R[继续下一轮循环]
    R --> E
    
    subgraph "Event Loop 核心"
        E
        F
        H
        I
        J
        K
        L
        N
        P
        Q
    end
    
    subgraph "mio 系统调用"
        S
        T
    end
    
    E --> S
    S --> T
    
    style A fill:#e1f5fe
    style D fill:#f3e5f5
    style E fill:#ffebee
    style N fill:#c8e6c9
    style S fill:#fff3e0
```

## 9. Event Loop 与 Executor 交互时序图

```mermaid
sequenceDiagram
    participant EL as Event Loop
    participant MIO as mio 系统
    participant WK as Waker
    participant EQ as 就绪队列
    participant EX as Executor
    
    loop Event Loop 循环
        EL->>MIO: poll.poll(&mut events, None)
        MIO-->>EL: 返回就绪事件列表
        
        loop 处理每个事件
            EL->>EL: 从事件获取 Token(id)
            EL->>EL: 查找 wakers.get(&id)
            
            alt Waker 存在
                EL->>WK: waker.wake()
                WK->>EQ: 将 task_id 加入队列
                WK->>EX: thread.unpark() 唤醒
                EX-->>EL: 继续执行任务
            else Waker 不存在
                EL->>EL: 跳过此事件
            end
        end
    end
    
    note over EL: Event Loop 持续运行<br/>监听所有注册的 I/O 事件
    note over MIO: 使用 epoll/kqueue/select<br/>高效监听文件描述符
    note over WK: Waker 包含任务ID<br/>和 Executor 引用
```

## 10. Event Loop 核心代码流程图

```mermaid
graph TD
    A[fn event_loop] --> B[创建 Events 缓冲区]
    B --> C[开始无限循环]
    
    C --> D[poll.poll 阻塞等待]
    D --> E[系统返回就绪事件]
    E --> F[遍历 events.iter]
    
    F --> G[获取事件 Token]
    G --> H[提取任务 ID]
    H --> I[锁定 wakers HashMap]
    I --> J{找到对应 Waker?}
    
    J -->|是| K[调用 waker.wake]
    J -->|否| L[跳过此事件]
    
    K --> M[任务 ID 加入就绪队列]
    M --> N[唤醒 Executor 线程]
    N --> O[继续下一个事件]
    L --> O
    
    O --> P{还有更多事件?}
    P -->|是| F
    P -->|否| Q[继续下一轮循环]
    Q --> C
    
    subgraph "关键代码逻辑"
        R
        S
        T
        U
        V
        W
    end
    
    B --> R
    C --> S
    F --> T
    G --> U
    I --> V
    K --> W
    
    style A fill:#e1f5fe
    style D fill:#ffebee
    style K fill:#c8e6c9
    style R fill:#f0f0f0
    style S fill:#f0f0f0
    style T fill:#f0f0f0
    style U fill:#f0f0f0
    style V fill:#f0f0f0
    style W fill:#f0f0f0
```

## 11. 任务状态转换图

```mermaid
stateDiagram-v2
    [*] --> 就绪: spawn 任务
    就绪 --> 执行: Executor 取出任务
    执行 --> 等待: future.poll() 返回 NotReady
    等待 --> 就绪: waker.wake() 被调用
    执行 --> 完成: future.poll() 返回 Ready
    完成 --> [*]
    
    等待 --> 重新执行: 任务重新轮询
    重新执行 --> 等待: 仍然 NotReady
    重新执行 --> 完成: 现在 Ready
    
    note right of 就绪
        任务在 ready_queue 中
        等待 Executor 处理
    end note
    
    note right of 执行
        Executor 调用
        future.poll(waker)
    end note
    
    note right of 等待
        任务注册到 Reactor
        等待 I/O 事件
    end note
```

这些 Mermaid 图表展示了 b-reactor-executor 项目的完整架构和执行流程，包括：

1. **原始单线程架构** - 展示主线程和Reactor线程的交互
2. **高级多线程架构** - 展示多个Executor和Reactor的并行执行
3. **核心组件交互** - 展示各组件间的关系
4. **数据流** - 展示数据在系统中的流动
5. **时序图** - 展示异步任务的执行时序
6. **系统架构概览** - 整体架构视图
7. **Waker 生命周期** - Waker 的创建、注册、唤醒过程
8. **任务状态转换** - 任务在不同状态间的转换

你可以直接复制这些 Mermaid 代码到支持 Mermaid 的编辑器中查看渲染结果！
