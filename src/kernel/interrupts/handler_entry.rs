/// x86_32 上的中断入口
/// 针对异常统一压入一个错误码,使中断函数签名统一
///
use crate::kernel::interrupts::ENTRY_SIZE;
use core::arch::{asm, global_asm};

/// 中处理函数类型
pub type InterruptHandler = fn(u32, u32);
/// 中断入口类型(封装iret)
pub type InterruptEntry = unsafe extern "C" fn();

/// 中断函数入口
#[no_mangle]
pub static INTERRUPT_HANDLER_ENTRY_TABLE: [InterruptEntry; ENTRY_SIZE] = [
    interrupt_handler_0x00,
    interrupt_handler_0x01,
    interrupt_handler_0x02,
    interrupt_handler_0x03,
    interrupt_handler_0x04,
    interrupt_handler_0x05,
    interrupt_handler_0x06,
    interrupt_handler_0x07,
    interrupt_handler_0x08,
    interrupt_handler_0x09,
    interrupt_handler_0x0a,
    interrupt_handler_0x0b,
    interrupt_handler_0x0c,
    interrupt_handler_0x0d,
    interrupt_handler_0x0e,
    interrupt_handler_0x0f,
    interrupt_handler_0x10,
    interrupt_handler_0x11,
    interrupt_handler_0x12,
    interrupt_handler_0x13,
    interrupt_handler_0x14,
    interrupt_handler_0x15,
    interrupt_handler_0x16,
    interrupt_handler_0x17,
    interrupt_handler_0x18,
    interrupt_handler_0x19,
    interrupt_handler_0x1a,
    interrupt_handler_0x1b,
    interrupt_handler_0x1c,
    interrupt_handler_0x1d,
    interrupt_handler_0x1e,
    interrupt_handler_0x1f,
    interrupt_handler_0x20,
    interrupt_handler_0x21,
    interrupt_handler_0x22,
    interrupt_handler_0x23,
    interrupt_handler_0x24,
    interrupt_handler_0x25,
    interrupt_handler_0x26,
    interrupt_handler_0x27,
    interrupt_handler_0x28,
    interrupt_handler_0x29,
    interrupt_handler_0x2a,
    interrupt_handler_0x2b,
    interrupt_handler_0x2c,
    interrupt_handler_0x2d,
    interrupt_handler_0x2e,
    interrupt_handler_0x2f,
];

// 中断入口宏
macro_rules! interrupt_handler {
    // 没有错误码,压入固定的错误码
    ($vector:expr, $name:ident, false) => {
        #[naked]
        #[no_mangle]
        unsafe extern "C" fn $name() {
            asm!(
                "push 0x20230612",
                "push {0}",
                "jmp interrupt_entry",
                const $vector,
                options(noreturn)
            );
        }
    };
    ($vector:expr, $name:ident, true) => {
        #[naked]
        #[no_mangle]
        unsafe extern "C" fn $name() {
            asm!(
                "push {0}",
                "jmp interrupt_entry",
                const $vector,
                options(noreturn)
            );
        }
    };
}

// 中断处理的统一逻辑
global_asm!(
    r#"
    .section .text
    .global interrupt_entry
interrupt_entry:
    mov eax, [esp]
    call [INTERRUPT_HANDLER_TABLE + eax * 4]
    xchg bx, bx
    add esp, 8
    iretd
"#
);

// 中断入口函数生成
interrupt_handler!(0x00, interrupt_handler_0x00, false); // divide by zero
interrupt_handler!(0x01, interrupt_handler_0x01, false); // debug
interrupt_handler!(0x02, interrupt_handler_0x02, false); // non maskable interrupt
interrupt_handler!(0x03, interrupt_handler_0x03, false); // breakpoint

interrupt_handler!(0x04, interrupt_handler_0x04, false); // overflow
interrupt_handler!(0x05, interrupt_handler_0x05, false); // bound range exceeded
interrupt_handler!(0x06, interrupt_handler_0x06, false); // invalid opcode
interrupt_handler!(0x07, interrupt_handler_0x07, false); // device not avilable

interrupt_handler!(0x08, interrupt_handler_0x08, true); // double fault
interrupt_handler!(0x09, interrupt_handler_0x09, false); // coprocessor segment overrun
interrupt_handler!(0x0a, interrupt_handler_0x0a, true); // invalid TSS
interrupt_handler!(0x0b, interrupt_handler_0x0b, true); // segment not present

interrupt_handler!(0x0c, interrupt_handler_0x0c, true); // stack segment fault
interrupt_handler!(0x0d, interrupt_handler_0x0d, true); // general protection fault
interrupt_handler!(0x0e, interrupt_handler_0x0e, true); // page fault
interrupt_handler!(0x0f, interrupt_handler_0x0f, false); // reserved

interrupt_handler!(0x10, interrupt_handler_0x10, false); // x87 floating point exception
interrupt_handler!(0x11, interrupt_handler_0x11, true); // alignment check
interrupt_handler!(0x12, interrupt_handler_0x12, false); // machine check
interrupt_handler!(0x13, interrupt_handler_0x13, false); // SIMD Floating - Point Exception

interrupt_handler!(0x14, interrupt_handler_0x14, false); // Virtualization Exception
interrupt_handler!(0x15, interrupt_handler_0x15, true); // Control Protection Exception
interrupt_handler!(0x16, interrupt_handler_0x16, false); // reserved
interrupt_handler!(0x17, interrupt_handler_0x17, false); // reserved

interrupt_handler!(0x18, interrupt_handler_0x18, false); // reserved
interrupt_handler!(0x19, interrupt_handler_0x19, false); // reserved
interrupt_handler!(0x1a, interrupt_handler_0x1a, false); // reserved
interrupt_handler!(0x1b, interrupt_handler_0x1b, false); // reserved

interrupt_handler!(0x1c, interrupt_handler_0x1c, false); // reserved
interrupt_handler!(0x1d, interrupt_handler_0x1d, false); // reserved
interrupt_handler!(0x1e, interrupt_handler_0x1e, false); // reserved
interrupt_handler!(0x1f, interrupt_handler_0x1f, false); // reserved

// 外中断
interrupt_handler!(0x20, interrupt_handler_0x20, false); // clock
interrupt_handler!(0x21, interrupt_handler_0x21, false);
interrupt_handler!(0x22, interrupt_handler_0x22, false);
interrupt_handler!(0x23, interrupt_handler_0x23, false);
interrupt_handler!(0x24, interrupt_handler_0x24, false);
interrupt_handler!(0x25, interrupt_handler_0x25, false);
interrupt_handler!(0x26, interrupt_handler_0x26, false);
interrupt_handler!(0x27, interrupt_handler_0x27, false);
interrupt_handler!(0x28, interrupt_handler_0x28, false);
interrupt_handler!(0x29, interrupt_handler_0x29, false);
interrupt_handler!(0x2a, interrupt_handler_0x2a, false);
interrupt_handler!(0x2b, interrupt_handler_0x2b, false);
interrupt_handler!(0x2c, interrupt_handler_0x2c, false);
interrupt_handler!(0x2d, interrupt_handler_0x2d, false);
interrupt_handler!(0x2e, interrupt_handler_0x2e, false);
interrupt_handler!(0x2f, interrupt_handler_0x2f, false);
