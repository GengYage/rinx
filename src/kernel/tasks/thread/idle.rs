use crate::info;
use crate::kernel::interrupts::enable_interrupt;
use crate::kernel::system_call::sys_call::sys_yield;
use core::arch::asm;

pub(crate) fn idle() -> u32 {
    enable_interrupt(true);

    let mut count: u64 = 0;

    loop {
        count += 1;
        info!("idle task, count:{}", count);
        // 开中断,停机CPU,等待外中断
        unsafe { asm!("sti", "hlt", options(nomem, nostack)) }

        // 调度到其他线程
        sys_yield();
    }
}