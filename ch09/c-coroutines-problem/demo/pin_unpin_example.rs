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
    println!("Unpin 的核心特性：即使被 Pin 包装，仍然可以安全地移动被包装的内容");
    
    // 展示 Unpin 类型的关键特性：可以安全地移动被 Pin 的内容
    println!("\n--- 方法1：使用 Pin::get_mut() 移动被 Pin 的内容 ---");
    
    let mut pinned_unpin = Box::pin(UnpinStruct {
        data: 42,
        name: "Hello".to_string(),
    });
    println!("[DEBUG] Pin 包装后，内容地址: {:p}", &*pinned_unpin);
    println!("[DEBUG] 原始内容: {:?}", &*pinned_unpin);
    
    // 关键：对于 Unpin 类型，我们可以安全地获取可变引用
    // 这实际上是在移动被 Pin 的内容！
    let unpin_ref = Pin::get_mut(Pin::as_mut(&mut pinned_unpin));
    println!("[DEBUG] 通过 Pin::get_mut() 获取的内容地址: {:p}", unpin_ref);
    unpin_ref.data = 100;
    println!("[DEBUG] 修改后的内容: {:?}", unpin_ref);
    
    // 展示 Unpin 类型可以安全地替换被 Pin 的内容
    println!("\n--- 方法2：使用 mem::replace 替换被 Pin 的内容 ---");
    
    use std::mem;
    let mut pinned_for_replace = Box::pin(UnpinStruct {
        data: 1,
        name: "Original".to_string(),
    });
    println!("[DEBUG] 替换前内容地址: {:p}", &*pinned_for_replace);
    println!("[DEBUG] 替换前内容: {:?}", &*pinned_for_replace);
    
    let new_value = UnpinStruct {
        data: 999,
        name: "Replaced".to_string(),
    };
    
    // 对于 Unpin 类型，我们可以安全地替换被 Pin 的内容
    let old_value = mem::replace(Pin::get_mut(Pin::as_mut(&mut pinned_for_replace)), new_value);
    println!("[DEBUG] 替换后内容地址: {:p}", &*pinned_for_replace);
    println!("[DEBUG] 被替换的旧值: {:?}", old_value);
    println!("[DEBUG] 新内容: {:?}", &*pinned_for_replace);
    
    // 展示 Unpin 类型可以安全地移动整个被 Pin 的内容
    println!("\n--- 方法3：移动整个被 Pin 的内容 ---");
    
    let pinned_for_move = Box::pin(UnpinStruct {
        data: 2,
        name: "ToMove".to_string(),
    });
    println!("[DEBUG] 移动前内容地址: {:p}", &*pinned_for_move);
    
    // 对于 Unpin 类型，我们可以安全地移动整个被 Pin 的内容
    // 使用 Pin::into_inner() 来移动被 Pin 的内容
    let moved_value = Pin::into_inner(pinned_for_move); // 这里移动了整个被 Pin 的内容！
    println!("[DEBUG] 移动后内容地址: {:p}", &moved_value);
    println!("[DEBUG] 移动后的内容: {:?}", moved_value);
    
    println!("\n[总结] Unpin 类型被 Pin 包装后，仍然可以安全地移动、替换、解包内容");
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
    println!("[DEBUG] 原始值地址: {:p}", &value);
    
    let mut pinned = Box::pin(value);
    println!("[DEBUG] Pin 包装后，Box 地址: {:p}", pinned.as_ref());
    println!("[DEBUG] Pin 包装后，内容地址: {:p}", &*pinned);
    
    // 初始化自引用
    SelfReferential::init(pinned.as_mut());
    println!("[DEBUG] 初始化自引用后，内容地址: {:p}", &*pinned);
    
    // 现在我们不能移动这个结构体，因为它是 !Unpin
    // let moved = pinned; // 这会导致编译错误
    
    // 但我们可以安全地访问它
    println!("数据: {}", SelfReferential::get_data(pinned.as_ref()));
    println!("自引用指针: {:?}", SelfReferential::get_self_ref(pinned.as_ref()));
    println!("[DEBUG] 自引用指针指向的地址: {:p}", &*pinned);
    
    // 我们可以移动整个 Pin，但不能移动被 Pin 的内容
    let _moved_pin = pinned; // 这是允许的
    println!("[DEBUG] 移动整个 Pin 后，内容地址保持不变");
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

// ===========================================
// 7. 移动对比示例
// ===========================================

fn move_comparison_example() {
    println!("\n=== Pin 内容移动对比示例 ===");
    
    // Unpin 类型：可以安全地移动被 Pin 的内容
    println!("--- Unpin 类型：可以安全地移动被 Pin 的内容 ---");
    let mut unpin_pinned = Box::pin(UnpinStruct {
        data: 1,
        name: "Unpin".to_string(),
    });
    println!("[DEBUG] Pin 包装前内容地址: {:p}", &*unpin_pinned);
    
    // 对于 Unpin 类型，我们可以安全地获取可变引用并修改
    let unpin_ref = Pin::get_mut(Pin::as_mut(&mut unpin_pinned));
    unpin_ref.data = 100;
    println!("[DEBUG] 修改后内容地址: {:p}", &*unpin_pinned);
    println!("[DEBUG] Unpin 类型：可以安全地修改被 Pin 的内容");
    
    // !Unpin 类型：不能安全地移动被 Pin 的内容
    println!("\n--- !Unpin 类型：不能安全地移动被 Pin 的内容 ---");
    let mut self_ref_pinned = Box::pin(SelfReferential::new(2));
    SelfReferential::init(self_ref_pinned.as_mut());
    println!("[DEBUG] Pin 包装后内容地址: {:p}", &*self_ref_pinned);
    println!("[DEBUG] 自引用指针: {:?}", SelfReferential::get_self_ref(self_ref_pinned.as_ref()));
    
    // 对于 !Unpin 类型，我们不能使用 Pin::get_mut() 来获取可变引用
    // 下面的代码会编译错误：
    // let self_ref_mut = Pin::get_mut(Pin::as_mut(&mut self_ref_pinned)); // 编译错误！
    
    // 我们只能通过 Pin 的方法来安全地访问
    let data = SelfReferential::get_data(self_ref_pinned.as_ref());
    println!("[DEBUG] 通过 Pin 安全访问数据: {}", data);
    println!("[DEBUG] !Unpin 类型：必须通过 Pin 的方法来安全访问，不能直接获取可变引用");
    
    println!("\n[总结] Unpin 允许移动被 Pin 的内容，!Unpin 不允许");
}

fn main() {
    println!("Pin 和 Unpin 示例\n");
    
    unpin_example();
    self_referential_example();
    move_comparison_example();
    future_pin_example();
    pin_manipulation_example();
    why_pin_needed();
    async_state_machine_example();
    
    // 演示 !Unpin 类型没有 Pin 保护时的移动行为
    dangerous_move_example();
    
    // 演示移动整个 Pin 的影响
    pin_move_impact_example();
    
    println!("\n=== 总结 ===");
    println!("• Unpin: 可以安全移动的类型 (大多数类型)");
    println!("• !Unpin: 不能安全移动的类型 (自引用结构等)");
    println!("• Pin: 防止被包装的值被移动");
    println!("• 在异步编程中，Future 通常需要被 Pin");
    println!("• Pin 提供了内存安全保证，防止悬空指针");
    println!("• !Unpin 类型没有 Pin 保护时，编译器允许移动，但后果自负");
}

/// 演示 !Unpin 类型没有 Pin 保护时的移动行为
fn dangerous_move_example() {
    println!("\n=== !Unpin 类型没有 Pin 保护时的移动 ===");
    
    // 创建一个 !Unpin 类型
    let dangerous = SelfReferential::new(42);
    
    println!("[DEBUG] 移动前:");
    println!("  dangerous 地址: {:p}", &dangerous);
    println!("  dangerous.data: {}", dangerous.data);
    println!("  dangerous.self_ref: {:?}", dangerous.self_ref);
    
    // 编译器不会阻止这个移动！
    // 即使 SelfReferential 是 !Unpin，没有 Pin 保护时依然可以移动
    let moved = dangerous;
    
    println!("[DEBUG] 移动后:");
    println!("  moved 地址: {:p}", &moved);
    println!("  moved.data: {}", moved.data);
    println!("  moved.self_ref: {:?}", moved.self_ref);
    
    println!("✅ 编译器允许 !Unpin 类型的移动（没有 Pin 保护时）");
    println!("⚠️  但是，如果这个结构体有自引用指针，移动后就会变成悬空指针");
    println!("   这就是'后果自负'的含义");
    
    // 演示为什么需要 Pin 保护
    println!("\n--- 对比：使用 Pin 保护 ---");
    let dangerous2 = SelfReferential::new(100);
    let pinned = Box::pin(dangerous2);
    
    println!("[DEBUG] Pin 保护后:");
    println!("  pinned 地址: {:p}", &pinned);
    println!("  pinned.data: {}", pinned.data);
    
    // 现在尝试移动被 Pin 保护的内容
    println!("[DEBUG] 尝试移动被 Pin 保护的内容...");
    // let moved_pinned = *pinned; // ❌ 这会导致编译错误！
    // let moved_pinned = Pin::get_mut(Pin::as_mut(&mut pinned)); // ❌ 这也会导致编译错误！
    
    println!("✅ Pin 保护阻止了危险的移动，编译器会报错");
    println!("   这就是 Pin 的作用：防止 !Unpin 类型被意外移动");
    
    // 演示被 Pin 的内容也可以移动（但后果自负）
    println!("\n--- 被 Pin 的内容也可以移动（但后果自负） ---");
    let dangerous3 = SelfReferential::new(200);
    let pinned3 = Box::pin(dangerous3);
    
    println!("[DEBUG] 移动整个 Pin 前:");
    println!("  pinned3 地址: {:p}", &pinned3);
    println!("  pinned3.data: {}", pinned3.data);
    
    // 移动整个 Pin 是允许的
    let moved_pin = pinned3;
    println!("[DEBUG] 移动整个 Pin 后:");
    println!("  moved_pin 地址: {:p}", &moved_pin);
    println!("  moved_pin.data: {}", moved_pin.data);
    println!("✅ 移动整个 Pin 是安全的，因为 Pin 本身不包含自引用");
    
    // 但是，如果尝试移动被 Pin 包装的内容，就是危险的
    println!("\n[DEBUG] 尝试移动被 Pin 包装的内容...");
    // 通过 unsafe 代码可以绕过 Pin 的保护（但后果自负）
    let dangerous_content = unsafe { 
        std::ptr::read(&*moved_pin) // 危险！绕过 Pin 保护
    };
    println!("[DEBUG] 绕过 Pin 保护移动内容后:");
    println!("  dangerous_content 地址: {:p}", &dangerous_content);
    println!("  dangerous_content.data: {}", dangerous_content.data);
    println!("⚠️  这样做是危险的，因为绕过了 Pin 的保护！");
    println!("   如果 SelfReferential 有自引用指针，现在就是悬空指针");
}

/// 演示移动整个 Pin 的影响
fn pin_move_impact_example() {
    println!("\n=== 移动整个 Pin 的影响分析 ===");
    
    // 创建一个自引用结构并初始化
    let mut self_ref = SelfReferential::new(42);
    let pinned = Box::pin(self_ref);
    
    // 初始化自引用指针
    let mut pinned_mut = pinned;
    SelfReferential::init(Pin::as_mut(&mut pinned_mut));
    
    println!("[DEBUG] 初始化后:");
    println!("  pinned_mut 地址: {:p}", &pinned_mut);
    println!("  pinned_mut.data: {}", pinned_mut.data);
    println!("  pinned_mut.self_ref: {:?}", pinned_mut.self_ref);
    
    // 移动整个 Pin
    let moved_pin = pinned_mut;
    
    println!("\n[DEBUG] 移动整个 Pin 后:");
    println!("  moved_pin 地址: {:p}", &moved_pin);
    println!("  moved_pin.data: {}", moved_pin.data);
    println!("  moved_pin.self_ref: {:?}", moved_pin.self_ref);
    
    // 检查自引用指针是否仍然有效
    if let Some(ptr) = moved_pin.self_ref {
        println!("[DEBUG] 检查自引用指针的有效性...");
        unsafe {
            let data = ptr.as_ref().data;
            println!("[DEBUG] 通过自引用指针访问的 data: {}", data);
            println!("✅ 自引用指针仍然有效！");
        }
    }
    
    println!("\n=== 分析结果 ===");
    println!("1. Pin 本身被移动了（地址从 {:p} 变为 {:p}）", &moved_pin, &moved_pin);
    println!("2. 但是被 Pin 包装的内容地址没有改变");
    println!("3. 自引用指针仍然指向正确的位置");
    println!("4. 因此移动整个 Pin 是安全的");
    
    // 对比：如果移动被 Pin 包装的内容会怎样
    println!("\n--- 对比：移动被 Pin 包装的内容 ---");
    let another_self_ref = SelfReferential::new(100);
    let another_pinned = Box::pin(another_self_ref);
    
    println!("[DEBUG] 另一个 Pin 包装的结构:");
    println!("  another_pinned 地址: {:p}", &another_pinned);
    println!("  another_pinned.data: {}", another_pinned.data);
    
    // 尝试移动被 Pin 包装的内容（危险）
    println!("[DEBUG] 尝试移动被 Pin 包装的内容...");
    let dangerous_move = unsafe { std::ptr::read(&*another_pinned) };
    
    println!("[DEBUG] 移动被 Pin 包装的内容后:");
    println!("  dangerous_move 地址: {:p}", &dangerous_move);
    println!("  dangerous_move.data: {}", dangerous_move.data);
    println!("⚠️  这样做是危险的，因为绕过了 Pin 的保护！");
    println!("   如果这个结构体有自引用指针，现在就是悬空指针");
    
    println!("\n=== 总结 ===");
    println!("• 移动整个 Pin：安全，因为 Pin 本身不包含自引用");
    println!("• 移动被 Pin 包装的内容：危险，因为绕过了 Pin 的保护");
    println!("• Pin 的作用是保护被包装的内容不被移动，而不是保护 Pin 本身");
}
