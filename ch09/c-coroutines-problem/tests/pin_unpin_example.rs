use std::pin::Pin;
use std::marker::PhantomPinned;
use std::ptr::NonNull;

// ===========================================
// 1. Unpin 类型示例
// ===========================================

#[derive(Debug)]
struct UnpinStruct {
    data: i32,
    name: String,
}

// 大多数类型都自动实现了 Unpin
// 这意味着它们可以安全地移动，即使被 Pin 包装

fn unpin_example() {
    println!("=== Unpin 示例 ===");
    
    let value = UnpinStruct {
        data: 42,
        name: "Hello".to_string(),
    };
    
    // 创建 Pin
    let pinned_value = Box::pin(value);
    
    // 即使被 Pin 包装，我们仍然可以移动整个 Pin
    let mut moved_pin = pinned_value;
    
    // 我们可以安全地获取可变引用
    let mut_ref = Pin::as_mut(&mut moved_pin);
    Pin::get_mut(mut_ref).data = 100;
    
    println!("Unpin 结构体: {:?}", moved_pin);
}

// ===========================================
// 2. !Unpin 类型示例 (自引用结构)
// ===========================================

#[derive(Debug)]
struct SelfReferential {
    data: i32,
    self_ref: Option<NonNull<SelfReferential>>,
    _pin: PhantomPinned, // 这个字段让结构体变成 !Unpin
}

impl SelfReferential {
    fn new(data: i32) -> Self {
        Self {
            data,
            self_ref: None,
            _pin: PhantomPinned,
        }
    }
    
    fn init(mut self: Pin<&mut Self>) {
        let this = unsafe { self.as_mut().get_unchecked_mut() };
        this.self_ref = Some(NonNull::from(&mut *this));
    }
    
    fn get_data(self: Pin<&Self>) -> i32 {
        self.data
    }
    
    fn get_self_ref(self: Pin<&Self>) -> Option<NonNull<SelfReferential>> {
        self.self_ref
    }
}

fn self_referential_example() {
    println!("\n=== !Unpin 示例 (自引用结构) ===");
    
    let value = SelfReferential::new(42);
    let mut pinned = Box::pin(value);
    
    // 初始化自引用
    SelfReferential::init(pinned.as_mut());
    
    // 现在我们不能移动这个结构体，因为它是 !Unpin
    // let moved = pinned; // 这会导致编译错误
    
    // 但我们可以安全地访问它
    println!("数据: {}", SelfReferential::get_data(pinned.as_ref()));
    println!("自引用指针: {:?}", SelfReferential::get_self_ref(pinned.as_ref()));
    
    // 我们可以移动整个 Pin，但不能移动被 Pin 的内容
    let _moved_pin = pinned; // 这是允许的
}

// ===========================================
// 3. Future 中的 Pin 使用
// ===========================================

use std::future::Future;
use std::task::{Context, Poll};

struct SimpleFuture {
    count: i32,
    max: i32,
}

impl Future for SimpleFuture {
    type Output = i32;
    
    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.count < self.max {
            self.count += 1;
            println!("计数: {}", self.count);
            Poll::Pending
        } else {
            Poll::Ready(self.count)
        }
    }
}

fn future_pin_example() {
    println!("\n=== Future 中的 Pin 使用 ===");
    
    let future = SimpleFuture { count: 0, max: 3 };
    let _pinned_future = Box::pin(future);
    
    // 在异步运行时中，Future 必须被 Pin
    // 这确保了 Future 在轮询过程中不会被移动
    println!("Future 被 Pin 包装，可以安全地轮询");
}

// ===========================================
// 4. Pin 的移动和获取引用
// ===========================================

fn pin_manipulation_example() {
    println!("\n=== Pin 操作示例 ===");
    
    let data = vec![1, 2, 3, 4, 5];
    let pinned = Box::pin(data);
    
    // 1. 获取不可变引用
    let immutable_ref = pinned.as_ref();
    println!("不可变引用: {:?}", immutable_ref);
    
    // 2. 获取可变引用 (需要 Pin<&mut T>)
    let mut pinned_mut = Box::pin(vec![6, 7, 8, 9, 10]);
    let mutable_ref = pinned_mut.as_mut();
    Pin::get_mut(mutable_ref).push(11);
    println!("可变引用修改后: {:?}", pinned_mut);
    
    // 3. 移动整个 Pin
    let moved_pin = pinned;
    println!("移动后的 Pin: {:?}", moved_pin);
}

// ===========================================
// 5. 为什么需要 Pin？
// ===========================================

fn why_pin_needed() {
    println!("\n=== 为什么需要 Pin？ ===");
    
    println!("1. 防止自引用结构被移动");
    println!("2. 确保 Future 在轮询过程中保持稳定");
    println!("3. 防止悬空指针和内存安全问题");
    println!("4. 支持异步编程中的状态机");
}

// ===========================================
// 6. 实际应用场景
// ===========================================

struct AsyncStateMachine {
    state: i32,
    _pin: PhantomPinned,
}

impl AsyncStateMachine {
    fn new() -> Self {
        Self {
            state: 0,
            _pin: PhantomPinned,
        }
    }
    
    fn step(self: Pin<&mut Self>) -> i32 {
        let this = unsafe { self.get_unchecked_mut() };
        this.state += 1;
        this.state
    }
}

fn async_state_machine_example() {
    println!("\n=== 异步状态机示例 ===");
    
    let mut machine = Box::pin(AsyncStateMachine::new());
    
    for i in 0..5 {
        let state = AsyncStateMachine::step(machine.as_mut());
        println!("步骤 {}: 状态 = {}", i + 1, state);
    }
}

fn main() {
    println!("Pin 和 Unpin 示例\n");
    
    unpin_example();
    self_referential_example();
    future_pin_example();
    pin_manipulation_example();
    why_pin_needed();
    async_state_machine_example();
    
    println!("\n=== 总结 ===");
    println!("• Unpin: 可以安全移动的类型 (大多数类型)");
    println!("• !Unpin: 不能安全移动的类型 (自引用结构等)");
    println!("• Pin: 防止被包装的值被移动");
    println!("• 在异步编程中，Future 通常需要被 Pin");
    println!("• Pin 提供了内存安全保证，防止悬空指针");
}
