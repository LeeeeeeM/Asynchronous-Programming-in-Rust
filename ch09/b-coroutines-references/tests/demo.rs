// 解释为什么 borrow_mut() 不需要 unsafe，而原始指针需要
// 运行: cargo test demo

use std::cell::RefCell;

#[test]
fn test_borrow_mut_vs_raw_pointer() {
    println!("\n=== RefMut vs 原始指针对比 ===");
    
    // RefMut 方式 - 不需要 unsafe
    let data = RefCell::new(42);
    let mut mut_ref = data.borrow_mut();  // 返回 RefMut<i32>
    
    println!("RefMut 方式:");
    println!("类型: RefMut<i32>");
    *mut_ref += 8;  // ✅ 不需要 unsafe
    println!("结果: {}", *mut_ref);
    
    // 原始指针方式 - 需要 unsafe
    let mut x = 42;
    let raw_ptr: *mut i32 = &mut x as *mut i32;
    
    println!("\n原始指针方式:");
    println!("类型: *mut i32");
    unsafe {
        *raw_ptr += 8;  // ❌ 需要 unsafe
    }
    println!("结果: {}", x);
    
    println!("\n关键区别:");
    println!("- RefMut<i32> 是智能指针，实现了 DerefMut trait");
    println!("- *mut i32 是原始指针，没有借用检查");
    println!("- *mut_ref += 1 实际上是 (*mut_ref.deref_mut()) += 1");
    println!("- *raw_ptr += 1 是直接解引用，需要 unsafe");
    
    assert_eq!(*mut_ref, 50);
    assert_eq!(x, 50);
}

#[test]
fn test_why_star_operator_needs_unsafe() {
    println!("\n=== 为什么 * 操作符需要 unsafe ===");
    
    let mut x = 42;
    let raw_ptr: *mut i32 = &mut x as *mut i32;
    
    println!("原始指针: *mut i32");
    println!("指针值: {:p}", raw_ptr);
    println!("指向的值: {}", x);
    
    // ❌ 这样写会编译错误
    println!("\n❌ 尝试直接使用 * 操作符:");
    println!("*raw_ptr += 1;  // 这会导致编译错误");
    println!("错误信息: dereference of raw pointer is unsafe and requires unsafe function or block");
    
    // ✅ 必须在 unsafe 块内
    println!("\n✅ 正确的做法 - 使用 unsafe 块:");
    unsafe {
        *raw_ptr += 1;  // 现在 x 是 43
        println!("unsafe 块内: *raw_ptr += 1");
        println!("修改后的值: {}", *raw_ptr);
    }
    
    println!("\n为什么需要 unsafe？");
    println!("1. 原始指针可能是 null");
    println!("2. 原始指针可能是悬垂指针（指向已释放的内存）");
    println!("3. 原始指针可能没有正确对齐");
    println!("4. 原始指针可能违反别名规则");
    println!("5. 原始指针可能导致数据竞争");
    
    println!("\n编译器无法检查这些风险，所以要求明确标记为 unsafe");
    println!("这是 Rust 的安全机制，防止未定义行为");
    
    assert_eq!(x, 43);
}


#[test]
fn test_why_self_reference_cannot_use_ref_mut() {
    println!("\n=== 为什么自引用不能用 &mut 而只能用 *mut ===");
    
    // 1. 尝试使用 &mut - 这会编译失败
    println!("1. 尝试使用 &mut 进行自引用:");
    println!("```rust");
    println!("struct BadStruct {{");
    println!("    buffer: String,");
    println!("    writer: Option<&mut String>,  // ❌ 编译错误");
    println!("}}");
    println!("```");
    
    // 2. 演示借用检查器的问题
    println!("\n2. 借用检查器的问题:");
    println!("当你试图同时访问 buffer 和 writer 时:");
    println!("- writer 持有对 buffer 的可变引用");
    println!("- 但 buffer 和 writer 在同一个结构体中");
    println!("- 这违反了'不能同时有可变借用和不可变借用'的规则");
    println!("- 编译器：❌ 无法表达这种关系");
    
    // 3. 生命周期问题
    println!("\n3. 生命周期问题:");
    println!("```rust");
    println!("struct BadStruct {{");
    println!("    buffer: String,");
    println!("    writer: Option<&'??? mut String>,  // 生命周期是什么？");
    println!("}}");
    println!("```");
    println!("- writer 的生命周期应该与 buffer 相同");
    println!("- 但 Rust 无法表达这种'指向自己的引用'");
    println!("- 编译器：❌ 生命周期无法表示");
    
    // 4. 移动语义问题
    println!("\n4. 移动语义问题:");
    println!("当结构体被移动时:");
    println!("- buffer 被移动到新位置");
    println!("- writer 中的引用变成悬垂指针");
    println!("- 这违反了 Rust 的内存安全保证");
    println!("- 编译器：❌ 移动后引用无效");
    
    // 5. 为什么 *mut 可以解决问题
    println!("\n5. 为什么 *mut 可以解决问题:");
    println!("```rust");
    println!("struct GoodStruct {{");
    println!("    buffer: Option<String>,");
    println!("    writer: Option<*mut String>,  // ✅ 可以");
    println!("}}");
    println!("```");
    
    println!("\n*mut 的优势:");
    println!("- 不携带生命周期信息");
    println!("- 绕过借用检查器");
    println!("- 可以指向同一结构体中的其他字段");
    println!("- 需要手动管理，但给了我们控制权");
    
    // 6. 实际演示
    println!("\n6. 实际演示:");
    
    // 使用原始指针实现自引用
    struct SelfRefStruct {
        buffer: Option<String>,
        writer: Option<*mut String>,
    }
    
    impl SelfRefStruct {
        fn new() -> Self {
            Self {
                buffer: None,
                writer: None,
            }
        }
        
        fn setup(&mut self) {
            // 创建数据
            self.buffer = Some(String::from("hello"));
            
            // 建立自引用关系
            if let Some(buffer) = self.buffer.as_mut() {
                self.writer = Some(buffer as *mut String);
            }
        }
        
        fn write(&mut self, content: &str) {
            if let Some(writer_ptr) = self.writer {
                unsafe {
                    let writer = &mut *writer_ptr;
                    writer.push_str(content);
                }
            }
        }
        
        fn get_content(&self) -> Option<&String> {
            self.buffer.as_ref()
        }
    }
    
    let mut s = SelfRefStruct::new();
    s.setup();
    s.write(" world");
    
    println!("自引用结构体结果: {}", s.get_content().unwrap());
    
    println!("\n总结:");
    println!("- &mut 有借用检查，无法表达自引用");
    println!("- *mut 无借用检查，可以表达自引用");
    println!("- 这是异步编程中实现状态机的必要手段");
    println!("- 虽然需要 unsafe，但在正确实现下是安全的");
    
    assert_eq!(s.get_content().unwrap(), "hello world");
}

#[test]
fn test_take_method_explanation() {
    println!("\n=== take() 方法详解 ===");
    
    // 1. 基本用法
    println!("1. 基本用法:");
    let mut option = Some(42);
    println!("初始: option = {:?}", option);
    
    let value = option.take();
    println!("take() 后: option = {:?}", option);
    println!("取出的值: value = {:?}", value);
    
    // 2. 在自引用结构体中的使用
    println!("\n2. 在自引用结构体中的使用:");
    
    struct ExampleStack {
        buffer: Option<String>,
        writer: Option<*mut String>,
    }
    
    let mut stack = ExampleStack {
        buffer: Some(String::from("hello")),
        writer: None,
    };
    
    // 建立自引用关系
    if let Some(buffer) = stack.buffer.as_mut() {
        stack.writer = Some(buffer as *mut String);
    }
    
    println!("建立关系后:");
    println!("buffer: {:?}", stack.buffer);
    println!("writer: {:?}", stack.writer);
    
    // 使用 take() 取出 writer 指针
    if let Some(writer_ptr) = stack.writer.take() {
        println!("\n使用 take() 取出 writer 指针:");
        println!("writer_ptr: {:p}", writer_ptr);
        println!("stack.writer 现在是: {:?}", stack.writer);
        
        // 使用指针
        unsafe {
            let writer = &mut *writer_ptr;
            writer.push_str(" world");
            println!("通过指针修改后的内容: {}", writer);
        }
        
        // 重新建立关系
        stack.writer = Some(writer_ptr);
        println!("重新建立关系后: {:?}", stack.writer);
    }
    
    // 3. 为什么使用 take()？
    println!("\n3. 为什么使用 take()？");
    println!("- take() 取出值并留下 None");
    println!("- 避免同时持有多个可变引用");
    println!("- 在状态转换时安全地转移所有权");
    println!("- 防止悬垂指针问题");
    
    // 4. 在你的代码中的具体作用
    println!("\n4. 在你的代码中的具体作用:");
    println!("```rust");
    println!("let writer = unsafe {{ &mut *self.stack.writer.take().unwrap() }};");
    println!("```");
    println!("- 取出 writer 指针");
    println!("- 将 stack.writer 设为 None");
    println!("- 避免同时持有 buffer 和 writer 的引用");
    println!("- 使用完后重新建立关系");
    
    // 5. 对比：不使用 take() 会怎样？
    println!("\n5. 对比：不使用 take() 会怎样？");
    println!("```rust");
    println!("// ❌ 这样会有借用冲突");
    println!("let writer = unsafe {{ &mut *self.stack.writer.unwrap() }};");
    println!("// 此时 stack.writer 仍然持有指针");
    println!("// 如果同时访问 stack.buffer，会有借用冲突");
    println!("```");
    
    println!("\n总结:");
    println!("- take() 是 Option 的方法，取出值并留下 None");
    println!("- 在自引用结构体中用于安全地转移所有权");
    println!("- 避免同时持有多个可变引用");
    println!("- 是解决借用检查器冲突的重要工具");
    
    assert_eq!(value, Some(42));
    assert_eq!(option, None);
}

#[test]
fn test_take_return_value_and_unwrap() {
    println!("\n=== take() 的返回值和 unwrap() 的作用 ===");
    
    // 1. Option<T> 的 take() 返回值
    println!("1. Option<T> 的 take() 返回值:");
    let mut option = Some(String::from("hello"));
    println!("初始: option = {:?}", option);
    
    let result = option.take();
    println!("take() 后: result = {:?}", result);
    println!("option 现在: {:?}", option);
    println!("result 的类型: Option<String>");
    
    // 2. 为什么需要 unwrap()？
    println!("\n2. 为什么需要 unwrap()？");
    println!("因为 take() 返回的是 Option<T>，不是 T！");
    
    // 直接使用会报错
    // let buffer: String = result;  // ❌ 类型不匹配
    
    // 需要 unwrap() 来取出值
    let buffer = result.unwrap();
    println!("unwrap() 后: buffer = {:?}", buffer);
    println!("buffer 的类型: String");
    
    // 3. 不同容器的 take() 返回值对比
    println!("\n3. 不同容器的 take() 返回值对比:");
    
    // Option<T>
    let mut opt = Some(42);
    let opt_result = opt.take();
    println!("Option<i32>::take() 返回: {:?} (类型: Option<i32>)", opt_result);
    
    // RefCell<T>
    use std::cell::RefCell;
    let cell = RefCell::new(42);
    let cell_result = cell.take();
    println!("RefCell<i32>::take() 返回: {} (类型: i32)", cell_result);
    println!("RefCell 的 take() 直接返回 T，不需要 unwrap()");
    
    // 4. 在你的代码中的应用
    println!("\n4. 在你的代码中的应用:");
    println!("```rust");
    println!("let buffer = self.stack.buffer.take().unwrap();");
    println!("//                    ^^^^^^^^^^^^ 返回 Option<String>");
    println!("//                                ^^^^^^^^ 取出 Some 中的值");
    println!("```");
    
    // 5. 分解步骤演示
    println!("\n5. 分解步骤演示:");
    let mut stack_buffer = Some(String::from("test"));
    println!("步骤1 - 初始: {:?}", stack_buffer);
    
    let step1 = stack_buffer.take();
    println!("步骤2 - take(): {:?}", step1);
    println!("步骤2 - stack_buffer: {:?}", stack_buffer);
    
    let step2 = step1.unwrap();
    println!("步骤3 - unwrap(): {:?}", step2);
    
    // 6. 错误示例
    println!("\n6. 错误示例:");
    println!("```rust");
    println!("let mut option = Some(42);");
    println!("let value: i32 = option.take();  // ❌ 类型不匹配");
    println!("// 错误: expected i32, found Option<i32>");
    println!("```");
    
    println!("\n总结:");
    println!("- Option<T> 的 take() 返回 Option<T>，需要 unwrap()");
    println!("- RefCell<T> 的 take() 直接返回 T，不需要 unwrap()");
    println!("- 不同容器的 take() 方法返回类型不同");
    println!("- unwrap() 用于从 Some(value) 中取出 value");
    
    assert_eq!(buffer, "hello");
    assert_eq!(opt_result, Some(42));
    assert_eq!(cell_result, 42);
}

#[test]
fn test_reference_to_pointer_vs_dereference() {
    println!("\n=== 引用转指针 vs 指针解引用 ===");
    
    // 1. 将引用转换为指针是安全的
    println!("1. 将引用转换为指针是安全的:");
    let mut x = 42;
    let reference = &mut x;                    // 安全的引用
    let pointer = reference as *mut i32;       // ✅ 安全的转换
    println!("reference: {:p}", reference);
    println!("pointer: {:p}", pointer);
    println!("转换是安全的，不需要 unsafe 块");
    
    // 2. 解引用指针是不安全的
    println!("\n2. 解引用指针是不安全的:");
    println!("```rust");
    println!("let value = *pointer;  // ❌ 编译错误：需要 unsafe");
    println!("```");
    
    // 正确的做法
    unsafe {
        let value = *pointer;  // ✅ 需要 unsafe 块
        println!("unsafe 块内解引用: {}", value);
    }
    
    // 3. 在你的代码中的应用
    println!("\n3. 在你的代码中的应用:");
    println!("```rust");
    println!("// 建立自引用关系 - 安全的转换");
    println!("self.stack.writer = Some(self.stack.buffer.as_mut().unwrap() as *mut String);");
    println!("//                    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^");
    println!("//                    引用转指针，不需要 unsafe");
    println!("");
    println!("// 使用指针 - 不安全的解引用");
    println!("let writer = unsafe {{ &mut *self.stack.writer.take().unwrap() }};");
    println!("//                    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^");
    println!("//                    指针解引用，需要 unsafe");
    println!("```");
    
    // 4. 为什么转换是安全的？
    println!("\n4. 为什么转换是安全的？");
    println!("- 引用转指针只是改变了类型，不涉及内存操作");
    println!("- 编译器知道引用的有效性");
    println!("- 转换过程没有风险");
    
    // 5. 为什么解引用不安全？
    println!("\n5. 为什么解引用不安全？");
    println!("- 指针可能指向无效内存");
    println!("- 指针可能为 null");
    println!("- 指针可能违反别名规则");
    println!("- 编译器无法检查这些风险");
    
    // 6. 实际演示
    println!("\n6. 实际演示:");
    let mut s = String::from("hello");
    println!("初始 String: {}", s);
    
    // 建立指针关系
    let s_ref = &mut s;
    let s_ptr = s_ref as *mut String;  // 安全的转换
    println!("引用地址: {:p}", s_ref);
    println!("指针地址: {:p}", s_ptr);
    
    // 使用指针（在 unsafe 块中）
    unsafe {
        let s_from_ptr = &mut *s_ptr;  // 不安全的解引用
        s_from_ptr.push_str(" world");
        println!("通过指针修改后: {}", s_from_ptr);
    }
    
    // 重新打印修改后的字符串
    println!("最终结果: {}", s);
    
    println!("\n总结:");
    println!("- 引用转指针: 安全，不需要 unsafe");
    println!("- 指针解引用: 不安全，需要 unsafe");
    println!("- 这是 Rust 安全模型的重要原则");
    println!("- 在你的代码中，建立自引用关系是安全的，使用指针需要 unsafe");
    
    assert_eq!(x, 42);
    assert_eq!(s, "hello world");
}
