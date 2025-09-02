mod advanced_runtime;

use advanced_runtime::demo_multi_executor_multi_reactor;

fn main() {
    println!("Rust 高级异步运行时系统演示");
    println!("================================\n");
    
    // 运行多Executor + 多Reactor架构演示
    demo_multi_executor_multi_reactor();
}
