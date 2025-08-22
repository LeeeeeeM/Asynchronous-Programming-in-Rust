use core::arch::asm;

const SSIZE: isize = 48;

#[derive(Debug, Default)]
#[repr(C)]
struct ThreadContext {
    s: u64,
    rsp: u64,
}

fn hello() -> ! {
    println!("I LOVE WAKING UP ON A NEW STACK!");
    loop {}
}

unsafe fn gt_switch(new: *const ThreadContext) {
    asm!(
        "mov rsp, [{0} + 0x08]",
        "ret",
        in(reg) new,
    );
}

fn main() {
    let mut ctx = ThreadContext::default();
    let mut stack = vec![0_u8; SSIZE as usize];

    unsafe {
        println!("{:p}", stack.as_mut_ptr());
        let stack_bottom = stack.as_mut_ptr().offset(SSIZE);
        println!("{:p}", stack_bottom);
        let sb_aligned = (stack_bottom as usize & !15) as *mut u8;
        println!("{:p}", sb_aligned);
        std::ptr::write(sb_aligned.offset(-16) as *mut u64, hello as u64);
        ctx.rsp = sb_aligned.offset(-16) as u64;
        gt_switch(&mut ctx);
    }
}
