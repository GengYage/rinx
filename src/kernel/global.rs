use crate::kernel::limit_of_type;
use crate::kernel::sync::mutex::Mutex;
use core::mem::size_of;
use lazy_static::lazy_static;
use x86::bits32::task::TaskStateSegment;
use x86::dtables::{lgdt, sgdt, DescriptorTablePointer};
use x86::segmentation::GateDescriptorBuilder;
use x86::segmentation::{
    BuildDescriptor, CodeSegmentType, DataSegmentType, Descriptor,
    DescriptorBuilder, SegmentDescriptorBuilder, SegmentSelector,
};
use x86::task::load_tr;
use x86::Ring::{Ring0, Ring3};

const GDT_SIZE: usize = 128;
/// 内核代码段全局描述符表索引
const KERNEL_CODE_IDX: usize = 1;
/// 内核数据段全局描述符表索引
const KERNEL_DATA_IDX: usize = 2;

/// 内核TSS描述符索引
const KERNEL_TSS_IDX: usize = 3;

/// 用户代码段全局描述符表索引
const USER_CODE_IDX: usize = 4;
/// 用户数据段全局描述符表索引
const USER_DATA_IDX: usize = 5;

/// TSS描述符
static mut TSS: TaskStateSegment = TaskStateSegment::new();

// 内核全局描述符
lazy_static! {
    static ref GDT: Mutex<[Descriptor; GDT_SIZE]> = {
        #[allow(unused_mut)]
        let mut gdt = Mutex::new([Descriptor::default(); GDT_SIZE]);
        gdt
    };
}

#[no_mangle]
pub fn init_gdt() {
    let mut gdtr: DescriptorTablePointer<Descriptor> = Default::default();
    unsafe {
        sgdt(&mut gdtr);
    }

    // 内核代码段
    let mut gdt_guard = GDT.lock();

    // 内核代码段
    gdt_guard[KERNEL_CODE_IDX] = DescriptorBuilder::code_descriptor(
        0,                            // 描述的内存起始位置
        0xffff,                       // 结束位置
        CodeSegmentType::ExecuteRead, // 0b1010 代码段/非依从/可读/没有被访问过
    )
    .limit_granularity_4kb() // 4k
    .db() // 32位
    .present() // 在内存中
    .dpl(Ring0) // 0特权级
    .finish();

    // 内核数据段
    gdt_guard[KERNEL_DATA_IDX] = DescriptorBuilder::data_descriptor(
        0,                          // 描述的内存起始位置
        0xffff,                     // 结束位置
        DataSegmentType::ReadWrite, // 0b0010 数据段/向上增长/可写/没有被访问过
    )
    .limit_granularity_4kb() // 4k
    .db() // 32位
    .present() // 在内存中
    .dpl(Ring0) // 0特权级
    .finish();

    // 用户代码段
    gdt_guard[USER_CODE_IDX] = DescriptorBuilder::code_descriptor(
        0,                            // 描述的内存起始位置
        0xffff,                       // 结束位置
        CodeSegmentType::ExecuteRead, // 0b1010 代码段/非依从/可读/没有被访问过
    )
    .limit_granularity_4kb() // 4k
    .db() // 32位
    .present() // 在内存中
    .dpl(Ring3) // 3特权级
    .finish();

    //  用户数据段
    gdt_guard[USER_DATA_IDX] = DescriptorBuilder::data_descriptor(
        0,                          // 描述的内存起始位置
        0xffff,                     // 结束位置
        DataSegmentType::ReadWrite, // 0b0010 数据段/向上增长/可写/没有被访问过
    )
    .limit_granularity_4kb() // 4k
    .db() // 32位
    .present() // 在内存中
    .dpl(Ring3) // 3特权级
    .finish();

    gdtr.base = gdt_guard.as_ptr();
    gdtr.limit = limit_of_type::<[Descriptor; GDT_SIZE]>();

    unsafe {
        lgdt(&gdtr);
    }
}

/// 初始化tss
#[no_mangle]
pub fn init_tss() {
    unsafe {
        // 0特权级别数据段 使用内核数据段描述符,请求特级0
        TSS.ss0 = SegmentSelector::new(KERNEL_DATA_IDX as u16, Ring0).bits();
    }

    GDT.lock()[KERNEL_TSS_IDX] =
        <DescriptorBuilder as GateDescriptorBuilder<u32>>::tss_descriptor(
            unsafe { &TSS as *const TaskStateSegment as u64 },
            size_of::<TaskStateSegment>() as u64,
            true,
        )
        .present() // 存在内存
        .dpl(Ring0) // 特权级0
        .finish();

    unsafe {
        load_tr(SegmentSelector::new(KERNEL_TSS_IDX as u16, Ring0));
    }
}
