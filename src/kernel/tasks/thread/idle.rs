use crate::kernel::interrupts::enable_interrupt;
use crate::kernel::system_call::sys_call::sys_yield;
use crate::printk;
use core::arch::asm;

pub(crate) fn idle() -> ! {
    enable_interrupt(true);

    loop {
        // 开中断,停机CPU,等待外中断
        unsafe { asm!("sti", "hlt", options(nomem, nostack)) }
        printk!(".");

        // 调度到其他线程
        sys_yield();
    }
}
