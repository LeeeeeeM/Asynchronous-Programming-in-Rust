use std::{
    collections::HashMap,
    sync::{Arc, Mutex, OnceLock},
    thread,
    time::Duration,
};

// ==================== 类型定义 ====================

pub type ExecutorId = usize;
pub type ReactorType = &'static str;
pub type TaskId = usize;

// ==================== Executor 相关 ====================

pub trait TaskExecutor: Send + Sync {
    fn wake_task(&self, task_id: TaskId);
    fn get_name(&self) -> String;
}

pub struct Executor {
    id: ExecutorId,
    name: String,
    ready_queue: Arc<Mutex<Vec<TaskId>>>,
    tasks: Arc<Mutex<HashMap<TaskId, Box<dyn Future<Output = String> + Send>>>>,
    next_task_id: Arc<Mutex<TaskId>>,
}

impl Executor {
    pub fn new(id: ExecutorId, name: String) -> Self {
        Self {
            id,
            name,
            ready_queue: Arc::new(Mutex::new(Vec::new())),
            tasks: Arc::new(Mutex::new(HashMap::new())),
            next_task_id: Arc::new(Mutex::new(0)),
        }
    }

    pub fn spawn<F>(&self, future: F) -> TaskId
    where
        F: Future<Output = String> + Send + 'static,
    {
        let task_id = {
            let mut next_id = self.next_task_id.lock().unwrap();
            *next_id += 1;
            *next_id
        };

        self.tasks.lock().unwrap().insert(task_id, Box::new(future));
        self.ready_queue.lock().unwrap().push(task_id);
        task_id
    }

    pub fn run(&self) {
        println!("{}: 开始运行", self.name);
        
        loop {
            // 处理就绪队列中的任务
            while let Some(task_id) = self.pop_ready() {
                if let Some(mut future) = self.get_future(task_id) {
                    let waker = self.create_waker(task_id);
                    
                    match future.poll(&waker) {
                        PollState::Ready(result) => {
                            println!("{}: 任务 {} 完成，结果: {}", self.name, task_id, result);
                        }
                        PollState::NotReady => {
                            // 重新插入任务列表
                            self.tasks.lock().unwrap().insert(task_id, future);
                        }
                    }
                }
            }

            // 检查是否还有任务
            let task_count = self.tasks.lock().unwrap().len();
            if task_count > 0 {
                println!("{}: 还有 {} 个待处理任务，等待唤醒...", self.name, task_count);
                thread::sleep(Duration::from_millis(100)); // 简化版本，实际应该用条件变量
            } else {
                println!("{}: 所有任务完成", self.name);
                break;
            }
        }
    }

    fn pop_ready(&self) -> Option<TaskId> {
        self.ready_queue.lock().unwrap().pop()
    }

    fn get_future(&self, task_id: TaskId) -> Option<Box<dyn Future<Output = String> + Send>> {
        self.tasks.lock().unwrap().remove(&task_id)
    }

    fn create_waker(&self, task_id: TaskId) -> Waker {
        Waker {
            task_id,
            executor_id: self.id,
        }
    }
}

impl TaskExecutor for Executor {
    fn wake_task(&self, task_id: TaskId) {
        println!("{}: 唤醒任务 {}", self.name, task_id);
        self.ready_queue.lock().unwrap().push(task_id);
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }
}

// ==================== Reactor 相关 ====================

pub trait Reactor: Send + Sync {
    fn reactor_type(&self) -> ReactorType;
    fn start(&self);
    fn register_waker(&self, task_id: TaskId, waker: Waker);
    fn deregister_waker(&self, task_id: TaskId);
}

pub struct NetworkReactor {
    reactor_type: ReactorType,
    wakers: Arc<Mutex<HashMap<TaskId, Waker>>>,
    running: Arc<Mutex<bool>>,
}

impl NetworkReactor {
    pub fn new() -> Self {
        Self {
            reactor_type: "network",
            wakers: Arc::new(Mutex::new(HashMap::new())),
            running: Arc::new(Mutex::new(false)),
        }
    }

    fn simulate_network_event(&self) {
        // 模拟网络事件
        thread::sleep(Duration::from_millis(200));
        
        let wakers = self.wakers.lock().unwrap();
        for (task_id, waker) in wakers.iter() {
            println!("网络Reactor: 检测到网络事件，唤醒任务 {}", task_id);
            waker.wake();
        }
    }
}

impl Reactor for NetworkReactor {
    fn reactor_type(&self) -> ReactorType {
        self.reactor_type
    }

    fn start(&self) {
        let running = self.running.clone();
        let wakers = self.wakers.clone();
        
        *running.lock().unwrap() = true;
        
        thread::spawn(move || {
            println!("网络Reactor: 开始监听网络事件");
            while *running.lock().unwrap() {
                // 模拟网络事件监听
                thread::sleep(Duration::from_millis(500));
                
                // 如果有注册的waker，模拟事件触发
                if !wakers.lock().unwrap().is_empty() {
                    let wakers_clone = wakers.clone();
                    thread::spawn(move || {
                        thread::sleep(Duration::from_millis(100));
                        let wakers = wakers_clone.lock().unwrap();
                        for (task_id, waker) in wakers.iter() {
                            println!("网络Reactor: 触发网络事件，唤醒任务 {}", task_id);
                            waker.wake();
                        }
                    });
                }
            }
        });
    }

    fn register_waker(&self, task_id: TaskId, waker: Waker) {
        println!("网络Reactor: 注册任务 {} 的waker", task_id);
        self.wakers.lock().unwrap().insert(task_id, waker);
    }

    fn deregister_waker(&self, task_id: TaskId) {
        println!("网络Reactor: 注销任务 {} 的waker", task_id);
        self.wakers.lock().unwrap().remove(&task_id);
    }
}

pub struct FileReactor {
    reactor_type: ReactorType,
    wakers: Arc<Mutex<HashMap<TaskId, Waker>>>,
    running: Arc<Mutex<bool>>,
}

impl FileReactor {
    pub fn new() -> Self {
        Self {
            reactor_type: "file",
            wakers: Arc::new(Mutex::new(HashMap::new())),
            running: Arc::new(Mutex::new(false)),
        }
    }
}

impl Reactor for FileReactor {
    fn reactor_type(&self) -> ReactorType {
        self.reactor_type
    }

    fn start(&self) {
        let running = self.running.clone();
        let wakers = self.wakers.clone();
        
        *running.lock().unwrap() = true;
        
        thread::spawn(move || {
            println!("文件Reactor: 开始监听文件系统事件");
            while *running.lock().unwrap() {
                thread::sleep(Duration::from_millis(800));
                
                if !wakers.lock().unwrap().is_empty() {
                    let wakers_clone = wakers.clone();
                    thread::spawn(move || {
                        thread::sleep(Duration::from_millis(150));
                        let wakers = wakers_clone.lock().unwrap();
                        for (task_id, waker) in wakers.iter() {
                            println!("文件Reactor: 触发文件事件，唤醒任务 {}", task_id);
                            waker.wake();
                        }
                    });
                }
            }
        });
    }

    fn register_waker(&self, task_id: TaskId, waker: Waker) {
        println!("文件Reactor: 注册任务 {} 的waker", task_id);
        self.wakers.lock().unwrap().insert(task_id, waker);
    }

    fn deregister_waker(&self, task_id: TaskId) {
        println!("文件Reactor: 注销任务 {} 的waker", task_id);
        self.wakers.lock().unwrap().remove(&task_id);
    }
}

pub struct TimerReactor {
    reactor_type: ReactorType,
    wakers: Arc<Mutex<HashMap<TaskId, Waker>>>,
    running: Arc<Mutex<bool>>,
}

impl TimerReactor {
    pub fn new() -> Self {
        Self {
            reactor_type: "timer",
            wakers: Arc::new(Mutex::new(HashMap::new())),
            running: Arc::new(Mutex::new(false)),
        }
    }
}

impl Reactor for TimerReactor {
    fn reactor_type(&self) -> ReactorType {
        self.reactor_type
    }

    fn start(&self) {
        let running = self.running.clone();
        let wakers = self.wakers.clone();
        
        *running.lock().unwrap() = true;
        
        thread::spawn(move || {
            println!("定时器Reactor: 开始监听定时器事件");
            while *running.lock().unwrap() {
                thread::sleep(Duration::from_millis(1000));
                
                if !wakers.lock().unwrap().is_empty() {
                    let wakers_clone = wakers.clone();
                    thread::spawn(move || {
                        thread::sleep(Duration::from_millis(50));
                        let wakers = wakers_clone.lock().unwrap();
                        for (task_id, waker) in wakers.iter() {
                            println!("定时器Reactor: 触发定时器事件，唤醒任务 {}", task_id);
                            waker.wake();
                        }
                    });
                }
            }
        });
    }

    fn register_waker(&self, task_id: TaskId, waker: Waker) {
        println!("定时器Reactor: 注册任务 {} 的waker", task_id);
        self.wakers.lock().unwrap().insert(task_id, waker);
    }

    fn deregister_waker(&self, task_id: TaskId) {
        println!("定时器Reactor: 注销任务 {} 的waker", task_id);
        self.wakers.lock().unwrap().remove(&task_id);
    }
}

// ==================== 管理器 ====================

#[derive(Clone)]
pub struct ExecutorRegistry {
    executors: Arc<Mutex<HashMap<ExecutorId, Arc<dyn TaskExecutor>>>>,
}

impl ExecutorRegistry {
    pub fn new() -> Self {
        Self {
            executors: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn register_executor(&self, id: ExecutorId, executor: Arc<dyn TaskExecutor>) {
        println!("注册执行器: {} (ID: {})", executor.get_name(), id);
        self.executors.lock().unwrap().insert(id, executor);
    }

    pub fn get_executor(&self, id: ExecutorId) -> Option<Arc<dyn TaskExecutor>> {
        self.executors.lock().unwrap().get(&id).cloned()
    }

    pub fn list_executors(&self) -> Vec<(ExecutorId, String)> {
        self.executors
            .lock()
            .unwrap()
            .iter()
            .map(|(id, executor)| (*id, executor.get_name()))
            .collect()
    }
}

#[derive(Clone)]
pub struct ReactorManager {
    reactors: Arc<Mutex<HashMap<ReactorType, Arc<dyn Reactor>>>>,
}

impl ReactorManager {
    pub fn new() -> Self {
        Self {
            reactors: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn register_reactor(&self, reactor_type: ReactorType, reactor: Arc<dyn Reactor>) {
        println!("注册Reactor: {} ({})", reactor_type, reactor.reactor_type());
        self.reactors.lock().unwrap().insert(reactor_type, reactor);
    }

    pub fn get_reactor(&self, reactor_type: ReactorType) -> Option<Arc<dyn Reactor>> {
        self.reactors.lock().unwrap().get(reactor_type).cloned()
    }

    pub fn start_all_reactors(&self) {
        println!("启动所有Reactor...");
        let reactors = self.reactors.lock().unwrap();
        for (reactor_type, reactor) in reactors.iter() {
            println!("启动Reactor: {}", reactor_type);
            reactor.start();
        }
    }

    pub fn list_reactors(&self) -> Vec<ReactorType> {
        self.reactors.lock().unwrap().keys().cloned().collect()
    }
}

// ==================== Waker 和 Future ====================

#[derive(Clone)]
pub struct Waker {
    task_id: TaskId,
    executor_id: ExecutorId,
}

impl Waker {
    pub fn wake(&self) {
        println!("Waker: 唤醒任务 {} (执行器 {})", self.task_id, self.executor_id);
        
        // 通过全局注册表找到对应的执行器
        if let Some(executor) = EXECUTOR_REGISTRY.get().unwrap().get_executor(self.executor_id) {
            executor.wake_task(self.task_id);
        } else {
            println!("警告: 找不到执行器 {}", self.executor_id);
        }
    }
}

pub trait Future {
    type Output;
    fn poll(&mut self, waker: &Waker) -> PollState<Self::Output>;
}

pub enum PollState<T> {
    Ready(T),
    NotReady,
}

// ==================== 示例 Future 实现 ====================

pub struct NetworkTask {
    task_id: TaskId,
    reactor_type: ReactorType,
    completed: bool,
}

impl NetworkTask {
    pub fn new(task_id: TaskId) -> Self {
        Self {
            task_id,
            reactor_type: "network",
            completed: false,
        }
    }
}

impl Future for NetworkTask {
    type Output = String;

    fn poll(&mut self, waker: &Waker) -> PollState<Self::Output> {
        if self.completed {
            PollState::Ready(format!("网络任务 {} 已完成", self.task_id))
        } else {
            // 注册到对应的Reactor
            if let Some(reactor) = REACTOR_MANAGER.get().unwrap().get_reactor(self.reactor_type) {
                reactor.register_waker(self.task_id, waker.clone());
            }
            self.completed = true;
            PollState::NotReady
        }
    }
}

pub struct FileTask {
    task_id: TaskId,
    reactor_type: ReactorType,
    completed: bool,
}

impl FileTask {
    pub fn new(task_id: TaskId) -> Self {
        Self {
            task_id,
            reactor_type: "file",
            completed: false,
        }
    }
}

impl Future for FileTask {
    type Output = String;

    fn poll(&mut self, waker: &Waker) -> PollState<Self::Output> {
        if self.completed {
            PollState::Ready(format!("文件任务 {} 已完成", self.task_id))
        } else {
            if let Some(reactor) = REACTOR_MANAGER.get().unwrap().get_reactor(self.reactor_type) {
                reactor.register_waker(self.task_id, waker.clone());
            }
            self.completed = true;
            PollState::NotReady
        }
    }
}

pub struct TimerTask {
    task_id: TaskId,
    reactor_type: ReactorType,
    completed: bool,
}

impl TimerTask {
    pub fn new(task_id: TaskId) -> Self {
        Self {
            task_id,
            reactor_type: "timer",
            completed: false,
        }
    }
}

impl Future for TimerTask {
    type Output = String;

    fn poll(&mut self, waker: &Waker) -> PollState<Self::Output> {
        if self.completed {
            PollState::Ready(format!("定时器任务 {} 已完成", self.task_id))
        } else {
            if let Some(reactor) = REACTOR_MANAGER.get().unwrap().get_reactor(self.reactor_type) {
                reactor.register_waker(self.task_id, waker.clone());
            }
            self.completed = true;
            PollState::NotReady
        }
    }
}

// ==================== 全局实例 ====================

static EXECUTOR_REGISTRY: OnceLock<ExecutorRegistry> = OnceLock::new();
static REACTOR_MANAGER: OnceLock<ReactorManager> = OnceLock::new();

pub fn init_advanced_runtime() -> (ExecutorRegistry, ReactorManager) {
    let executor_registry = ExecutorRegistry::new();
    let reactor_manager = ReactorManager::new();
    
    EXECUTOR_REGISTRY.set(executor_registry.clone()).ok();
    REACTOR_MANAGER.set(reactor_manager.clone()).ok();
    
    (executor_registry, reactor_manager)
}

// ==================== 演示函数 ====================

pub fn demo_multi_executor_multi_reactor() {
    println!("=== 多Executor + 多Reactor 架构演示 ===\n");
    
    // 初始化运行时
    let (executor_registry, reactor_manager) = init_advanced_runtime();
    
    // 注册不同类型的Reactor
    reactor_manager.register_reactor("network", Arc::new(NetworkReactor::new()));
    reactor_manager.register_reactor("file", Arc::new(FileReactor::new()));
    reactor_manager.register_reactor("timer", Arc::new(TimerReactor::new()));
    
    // 启动所有Reactor
    reactor_manager.start_all_reactors();
    
    // 创建多个Executor
    let network_executor = Arc::new(Executor::new(1, "网络执行器".to_string()));
    let file_executor = Arc::new(Executor::new(2, "文件执行器".to_string()));
    let timer_executor = Arc::new(Executor::new(3, "定时器执行器".to_string()));
    
    // 注册Executor
    executor_registry.register_executor(1, network_executor.clone());
    executor_registry.register_executor(2, file_executor.clone());
    executor_registry.register_executor(3, timer_executor.clone());
    
    // 显示注册信息
    println!("\n已注册的执行器:");
    for (id, name) in executor_registry.list_executors() {
        println!("  - {} (ID: {})", name, id);
    }
    
    println!("\n已注册的Reactor:");
    for reactor_type in reactor_manager.list_reactors() {
        println!("  - {}", reactor_type);
    }
    
    // 创建不同类型的任务
    println!("\n创建任务...");
    let network_task1 = NetworkTask::new(101);
    let network_task2 = NetworkTask::new(102);
    let file_task1 = FileTask::new(201);
    let file_task2 = FileTask::new(202);
    let timer_task1 = TimerTask::new(301);
    let timer_task2 = TimerTask::new(302);
    
    // 将任务分配给对应的Executor
    network_executor.spawn(network_task1);
    network_executor.spawn(network_task2);
    file_executor.spawn(file_task1);
    file_executor.spawn(file_task2);
    timer_executor.spawn(timer_task1);
    timer_executor.spawn(timer_task2);
    
    println!("\n开始执行任务...\n");
    
    // 在独立线程中运行每个Executor
    let network_executor_clone = network_executor.clone();
    let file_executor_clone = file_executor.clone();
    let timer_executor_clone = timer_executor.clone();
    
    thread::spawn(move || network_executor_clone.run());
    thread::spawn(move || file_executor_clone.run());
    thread::spawn(move || timer_executor_clone.run());
    
    // 等待一段时间让演示完成
    thread::sleep(Duration::from_secs(5));
    
    println!("\n=== 演示完成 ===");
}
