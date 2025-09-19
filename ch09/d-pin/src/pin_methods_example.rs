use std::pin::Pin;
use std::marker::PhantomPinned;

// ===========================================
// 示例1: Unpin 类型 - 可以安全移动
// ===========================================

#[derive(Debug)]
struct UnpinType {
    value: i32,
}

impl UnpinType {
    fn new(value: i32) -> Self {
        Self { value }
    }
    
    fn set_value(&mut self, value: i32) {
        self.value = value;
    }
}

// 注意：UnpinType 默认实现了 Unpin trait
// 这意味着它可以安全地移动

// ===========================================
// 示例2: !Unpin 类型 - 不能安全移动（自引用）
// ===========================================

#[derive(Debug)]
struct NotUnpinType {
    value: i32,
    self_ref: Option<*const i32>,  // 自引用指针
    _pin: PhantomPinned,           // 标记为 !Unpin
}

impl NotUnpinType {
    fn new(value: i32) -> Self {
        Self {
            value,
            self_ref: None,
            _pin: PhantomPinned,
        }
    }
    
    fn set_value(&mut self, value: i32) {
        self.value = value;
        // 设置自引用指针
        self.self_ref = Some(&self.value as *const i32);
    }
    
    fn get_self_ref(&self) -> Option<*const i32> {
        self.self_ref
    }
}

// 注意：NotUnpinType 被标记为 !Unpin
// 这意味着它不能被安全地移动

// ===========================================
// Pin 创建方法详解
// ===========================================

fn demonstrate_pin_creation_methods() {
    println!("=== Pin 创建方法详解 ===\n");
    
    // ===========================================
    // Unpin 类型的 Pin 创建方法
    // ===========================================
    
    println!("1. Unpin 类型的 Pin 创建方法:");
    println!("   Unpin 类型可以使用多种方法创建 Pin");
    
    let mut unpin_value = UnpinType::new(42);
    
    // 方法1: Pin::new() - 最常用
    let _pinned1 = Pin::new(&mut unpin_value);
    println!("   ✅ Pin::new(&mut value) - 最常用");
    
    // 方法2: Box::pin() - 也可以用于 Unpin 类型
    let unpin_value2 = UnpinType::new(100);
    let _pinned2 = Box::pin(unpin_value2);
    println!("   ✅ Box::pin(value) - 也可以用于 Unpin 类型");
    
    // 方法3: pin! 宏 - 需要 feature
    // let pinned3 = pin!(UnpinType::new(200));
    // println!("   ✅ pin!(value) - 需要 feature");
    
    // 方法4: Pin::new_unchecked() - unsafe
    unsafe {
        let _pinned4 = Pin::new_unchecked(&mut unpin_value);
        println!("   ✅ Pin::new_unchecked(&mut value) - unsafe 方法");
    }
    
    // ===========================================
    // !Unpin 类型的 Pin 创建方法
    // ===========================================
    
    println!("\n2. !Unpin 类型的 Pin 创建方法:");
    println!("   !Unpin 类型只能使用特定的方法创建 Pin");
    
    let mut not_unpin_value = NotUnpinType::new(200);
    not_unpin_value.set_value(300);
    
    // 方法1: Box::pin() - 最常用
    let _pinned1 = Box::pin(not_unpin_value);
    println!("   ✅ Box::pin(value) - 最常用，推荐方法");
    
    // 方法2: pin! 宏 - 需要 feature
    // let not_unpin_value2 = NotUnpinType::new(400);
    // let pinned2 = pin!(not_unpin_value2);
    // println!("   ✅ pin!(value) - 需要 feature");
    
    // 方法3: Pin::new_unchecked() - unsafe
    let mut not_unpin_value3 = NotUnpinType::new(500);
    not_unpin_value3.set_value(600);
    unsafe {
        let _pinned3 = Pin::new_unchecked(&mut not_unpin_value3);
        println!("   ✅ Pin::new_unchecked(&mut value) - unsafe 方法");
    }
    
    // ❌ 以下方法会导致编译错误
    // let mut not_unpin_value4 = NotUnpinType::new(700);
    // let pinned4 = Pin::new(&mut not_unpin_value4); // 编译错误！
    
    println!("\n3. 为什么 !Unpin 类型不能使用 Pin::new()?");
    println!("   Pin::new() 要求类型实现 Unpin trait");
    println!("   !Unpin 类型没有实现 Unpin trait");
    println!("   这是 Rust 的安全保证，防止自引用结构体被移动");
    
    println!("\n4. 各种方法的优缺点:");
    println!("   Pin::new():");
    println!("     - 优点: 安全，编译器保证");
    println!("     - 缺点: 只适用于 Unpin 类型");
    println!("     - 适用: 普通类型 (i32, String, Vec<T> 等)");
    
    println!("\n   Box::pin():");
    println!("     - 优点: 适用于所有类型，最常用");
    println!("     - 缺点: 需要堆分配");
    println!("     - 适用: 所有类型，特别是 !Unpin 类型");
    
    println!("\n   pin! 宏:");
    println!("     - 优点: 栈分配，性能好");
    println!("     - 缺点: 需要 feature，作用域限制");
    println!("     - 适用: 局部变量，需要栈分配时");
    
    println!("\n   Pin::new_unchecked():");
    println!("     - 优点: 灵活，适用于所有类型");
    println!("     - 缺点: unsafe，需要开发者保证安全");
    println!("     - 适用: 特殊场景，需要精确控制时");
}

// ===========================================
// 两种获取可变引用的方法
// ===========================================

fn demonstrate_pin_methods() {
    println!("=== Pin 的两种获取可变引用的方法 ===\n");
    
    // 创建 Unpin 类型的 Pin
    let mut unpin_value = UnpinType::new(42);
    let mut pinned_unpin = Pin::new(&mut unpin_value);
    
    // 创建 !Unpin 类型的 Pin - 使用 Box::pin
    let mut not_unpin_value = NotUnpinType::new(100);
    not_unpin_value.set_value(200);
    let mut pinned_not_unpin = Box::pin(not_unpin_value);
    
    println!("1. get_mut() - 用于 Unpin 类型");
    println!("   这是安全的方法，编译器会检查类型是否实现了 Unpin");
    
    // 方法1: get_mut() - 只对 Unpin 类型有效
    let mut_ref = pinned_unpin.as_mut().get_mut();
    println!("   ✅ Unpin 类型可以使用 get_mut()");
    mut_ref.set_value(999);
    println!("   修改后的值: {:?}", mut_ref);
    
    // 尝试对 !Unpin 类型使用 get_mut() 会失败
    // let mut_ref = pinned_not_unpin.get_mut(); // 这会导致编译错误
    
    println!("\n2. get_unchecked_mut() - 用于 !Unpin 类型");
    println!("   这是 unsafe 方法，需要开发者保证不会移动值");
    
    // 方法2: get_unchecked_mut() - 用于 !Unpin 类型
    unsafe {
        let mut_ref = pinned_not_unpin.as_mut().get_unchecked_mut();
        println!("   ✅ !Unpin 类型可以使用 get_unchecked_mut()");
        mut_ref.set_value(888);
        println!("   修改后的值: {:?}", mut_ref);
        println!("   自引用指针: {:?}", mut_ref.get_self_ref());
    }
    
    println!("\n注意: as_mut() 是转换方法，将 Pin<Box<T>> 转换为 Pin<&mut T>");
    println!("在实际使用中，我们通常使用 as_mut() 来获取可变引用");
    
    println!("\n3. 两种使用模式对比:");
    println!("   模式1: 先获取可变引用，再调用方法");
    println!("   let mut_ref = pinned.as_mut().get_mut();");
    println!("   mut_ref.set_value(999);  // set_value 接受 &mut self");
    
    println!("\n   模式2: 直接在 Pin<&mut T> 上调用方法");
    println!("   pinned.as_mut().poll(waker);  // poll 接受 Pin<&mut Self>");
    println!("   关键区别：方法签名不同！");
    println!("   - set_value(&mut self) 需要 &mut T");
    println!("   - poll(self: Pin<&mut Self>) 需要 Pin<&mut T>");
}

// ===========================================
// 实际应用场景：Future 实现
// ===========================================

trait Future {
    type Output;
    fn poll(self: Pin<&mut Self>) -> Self::Output;
}

struct MyFuture {
    value: i32,
    _pin: PhantomPinned,
}

impl MyFuture {
    fn new(value: i32) -> Self {
        Self {
            value,
            _pin: PhantomPinned,
        }
    }
    
    fn set_value(&mut self, value: i32) {
        self.value = value;
    }
}

impl Future for MyFuture {
    type Output = i32;
    
    fn poll(self: Pin<&mut Self>) -> Self::Output {
        // 在 Future 的 poll 方法中，我们需要获取可变引用
        // 由于 MyFuture 是 !Unpin 类型，我们使用 get_unchecked_mut
        
        let this = unsafe { self.get_unchecked_mut() };
        this.value += 1;
        this.value
    }
}

fn demonstrate_future_usage() {
    println!("\n=== 实际应用场景：Future 实现 ===\n");
    
    let future = MyFuture::new(10);
    let mut pinned_future = Box::pin(future);
    
    println!("初始值: {}", pinned_future.value);
    
    // 模式1: 先获取可变引用，再调用方法
    println!("\n模式1: 先获取可变引用，再调用方法");
    println!("原因: set_value(&mut self) 需要 &mut T");
    unsafe {
        let mut_ref = pinned_future.as_mut().get_unchecked_mut();
        println!("获取可变引用后，调用 set_value:");
        mut_ref.set_value(20);
        println!("修改后的值: {}", mut_ref.value);
    }
    
    // 模式2: 直接在 Pin<&mut T> 上调用方法
    println!("\n模式2: 直接在 Pin<&mut T> 上调用方法");
    println!("原因: poll(self: Pin<&mut Self>) 接受 Pin<&mut T>");
    let result1 = pinned_future.as_mut().poll();
    println!("第一次 poll 结果: {}", result1);
    
    let result2 = pinned_future.as_mut().poll();
    println!("第二次 poll 结果: {}", result2);
    
    println!("\n关键理解:");
    println!("- set_value(&mut self) → 需要 &mut T → 需要 get_unchecked_mut()");
    println!("- poll(self: Pin<&mut Self>) → 需要 Pin<&mut T> → 可以直接调用");
    println!("- 这就是为什么 f1.as_mut().poll(waker) 可以直接调用！");
}

// ===========================================
// 安全性和最佳实践
// ===========================================

fn demonstrate_safety_considerations() {
    println!("\n=== 安全性和最佳实践 ===\n");
    
    println!("1. get_mut() - 最安全");
    println!("   - 只对 Unpin 类型有效");
    println!("   - 编译器保证安全性");
    println!("   - 推荐用于普通类型");
    
    println!("\n2. get_unchecked_mut() - 用于 !Unpin 类型");
    println!("   - 需要类型被标记为 !Unpin");
    println!("   - 需要开发者保证不会移动值");
    println!("   - 推荐用于自引用结构体");
    
    println!("\n3. as_mut() - 转换方法");
    println!("   - 将 Pin<Box<T>> 转换为 Pin<&mut T>");
    println!("   - 然后可以使用 get_mut() 或 get_unchecked_mut()");
    println!("   - 推荐用于 Box::pin 创建的值");
    
    println!("\n4. 最佳实践：");
    println!("   - 优先使用 get_mut()");
    println!("   - 对于自引用结构体，使用 get_unchecked_mut()");
    println!("   - 使用 Box::pin 创建 !Unpin 类型的 Pin");
    println!("   - 确保 Pin 包装的值不会被移动");
}

// ===========================================
// 错误示例：展示常见错误
// ===========================================

fn demonstrate_common_errors() {
    println!("\n=== 常见错误示例 ===\n");
    
    println!("1. 尝试直接创建 !Unpin 类型的 Pin:");
    println!("   let mut not_unpin = NotUnpinType::new(42);");
    println!("   let pinned = Pin::new(&mut not_unpin); // ❌ 编译错误");
    println!("   错误原因：Pin::new() 要求类型实现 Unpin trait");
    
    println!("\n2. 尝试对 !Unpin 类型使用 get_mut():");
    println!("   let not_unpin = NotUnpinType::new(42);");
    println!("   let pinned = Box::pin(not_unpin);");
    println!("   let mut_ref = pinned.get_mut(); // ❌ 编译错误");
    println!("   错误原因：get_mut() 只适用于 Unpin 类型");
    
    println!("\n3. 正确的做法:");
    println!("   let not_unpin = NotUnpinType::new(42);");
    println!("   let pinned = Box::pin(not_unpin); // ✅ 正确");
    println!("   unsafe {{ pinned.get_unchecked_mut() }} // ✅ 正确");
}

// ===========================================
// 什么时候需要 get_unchecked_mut()？
// ===========================================

fn demonstrate_when_to_use_get_unchecked_mut() {
    println!("\n=== 什么时候需要 get_unchecked_mut()？ ===\n");
    
    let mut not_unpin = NotUnpinType::new(42);
    not_unpin.set_value(100);
    let mut pinned = Box::pin(not_unpin);
    
    println!("情况1: 需要访问字段 - 需要 get_unchecked_mut()");
    unsafe {
        let this = pinned.as_mut().get_unchecked_mut();
        println!("   访问字段: {}", this.value);  // 需要 get_unchecked_mut()
        this.value = 200;  // 修改字段 - 需要 get_unchecked_mut()
    }
    
    println!("\n情况2: 调用接受 &mut self 的方法 - 需要 get_unchecked_mut()");
    unsafe {
        let this = pinned.as_mut().get_unchecked_mut();
        this.set_value(300);  // set_value(&mut self) - 需要 get_unchecked_mut()
    }
    
    println!("\n情况3: 调用接受 Pin<&mut Self> 的方法 - 不需要 get_unchecked_mut()");
    // 注意：NotUnpinType 没有实现 Future，这里用 MyFuture 演示
    let future = MyFuture::new(50);
    let mut pinned_future = Box::pin(future);
    let result = pinned_future.as_mut().poll();  // poll(self: Pin<&mut Self>) - 直接调用
    println!("   poll 结果: {}", result);
    
    println!("\n情况4: 模式匹配 - 需要 get_unchecked_mut()");
    unsafe {
        let this = pinned.as_mut().get_unchecked_mut();
        match this.value {
            300 => println!("   值匹配: 300"),
            _ => println!("   其他值"),
        }
    }
    
    println!("\n总结：");
    println!("- 访问字段 → 需要 get_unchecked_mut()");
    println!("- 调用 &mut self 方法 → 需要 get_unchecked_mut()");
    println!("- 调用 Pin<&mut Self> 方法 → 直接调用");
    println!("- 模式匹配 → 需要 get_unchecked_mut()");
    println!("- 任何需要 &mut T 的操作 → 需要 get_unchecked_mut()");
}

fn main() {
    demonstrate_pin_creation_methods();
    demonstrate_pin_methods();
    demonstrate_future_usage();
    demonstrate_safety_considerations();
    demonstrate_common_errors();
    demonstrate_when_to_use_get_unchecked_mut();
}
