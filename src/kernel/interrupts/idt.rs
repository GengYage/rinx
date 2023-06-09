use crate::kernel::interrupts::entry::Entry;
use crate::kernel::interrupts::handler::INTERRUPT_HANDLER_TABLE;
use crate::kernel::interrupts::handler_entry::{
    InterruptEntry, INTERRUPT_HANDLER_ENTRY_TABLE,
};
use crate::kernel::interrupts::pic::handler::default_external_handler;
use crate::kernel::interrupts::{ENTRY_SIZE, IDT_SIZE, IRQ_MASTER_NR};
use crate::kernel::limit_of_type;
use crate::kernel::system_call::system_call;
use lazy_static::lazy_static;
use spin::Mutex;
use x86::dtables::{lidt, DescriptorTablePointer};

lazy_static! {
    pub static ref INTERRUPT_ENTRY: Mutex<[Entry<InterruptEntry>; IDT_SIZE]> = {
        #[allow(unused_mut)]
        let mut interrupt_entry_table: Mutex<
            [Entry<InterruptEntry>; IDT_SIZE],
        > = Mutex::new([Entry::missing(); IDT_SIZE]);

        interrupt_entry_table
    };
}

pub fn init_idt() {
    let mut idt: DescriptorTablePointer<Entry<InterruptEntry>> =
        DescriptorTablePointer::default();

    (0..ENTRY_SIZE).for_each(|index| {
        INTERRUPT_ENTRY.lock()[index]
            .set_handler_fn(INTERRUPT_HANDLER_ENTRY_TABLE[index]);
    });

    // 初始化外中断默认处理函数
    (IRQ_MASTER_NR..ENTRY_SIZE).for_each(|index| unsafe {
        INTERRUPT_HANDLER_TABLE[index] = default_external_handler;
    });

    // 初始化系统调用
    INTERRUPT_ENTRY.lock()[0x80].set_handler_fn(system_call);
    // 设置为用户态
    INTERRUPT_ENTRY.lock()[0x80].options.set_privilege_level(3);

    idt.base = INTERRUPT_ENTRY.lock().as_ptr();
    idt.limit = limit_of_type::<[Entry<InterruptEntry>; IDT_SIZE]>();

    unsafe {
        lidt(&idt);
    }
}
