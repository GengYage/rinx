[org 0x1000]

; 魔数
dw 0x55aa

; 打印字符串
mov si, loading
call print

detect_memory:
  xor ebx, ebx; 清空ebx

  ; 结构体缓存位置
  mov ax, 0
  mov es, ax
  mov edi, addrs_buffer

  mov edx, 0x534d4150; 固定签名 SMAP
  .next:
    ; 子功能号
    mov eax, 0xe820
    ; addrs 结构的大小
    mov ecx, 20

    ; 调用0x15系统调用
    int 0x15

    ; 如果CF置位,表示出错
    jc error

    ; 将缓存指针指向下一个
    add di, cx
    ; 将结构体数量+1
    inc word [addrs_count]

    ; 是0表示检测结束
    cmp ebx, 0
    ; 否则继续检测下一个
    jnz .next

    mov si, detecting
    call print

  ;   xchg bx, bx
  ;   
  ;   ; 结构体数量
  ;   mov cx, [addrs_count]
  ;   ; 结构体指针
  ;   mov si, 0
  ; .show:
  ;   ; 解析结构体,基地址低32位
  ;   ; 0-4低32位 4-8 高32位
  ;   mov eax, [addrs_buffer + si]
  ;   ; 内存长度
  ;   ; 8-12 内存长度低32位, 12-16内存长度高12位
  ;   mov ebx, [addrs_buffer + si + 8]
  ;   ; 内存类型
  ;   mov edx, [addrs_buffer + si + 16]
  ;   ; 指针+20 指向下一个结构体
  ;   add si, 20
  ;   xchg bx, bx
  ;   ; loop 是根据ecx的值循环
  ;   loop .show

; 进入保护模式的准备
jmp prepare_protected_mode

prepare_protected_mode:
  cli; 关闭中断

  ; 打开A20总线
  in al, 0x92
  or al, 0b10; 第二位置1
  out 0x92, al

  ; 加载gdt
  lgdt [gdt_ptr]

  ; 启动保护模式
  mov eax, cr0
  or eax, 1
  mov cr0, eax

  ; 用跳转来刷新保护模式
  jmp dword code_selector:protected_mode

print:
  mov ah, 0x0e
  .next:
    ; 获取当前字符
    mov al, [si]
    ; 比较当前字符是不是0
    cmp al, 0
    ; 是的话就代表已经到达字符串末尾,直接ret
    jz .done
    ; 否则,打印当前字符
    int 0x10
    ; 自增si,去打印下一个字符
    inc si
    ; 回到next继续打印
    jmp .next
  .done:
    ; 函数返回
    ret 

loading: 
  db "Loading Rnix...", 10, 13, 0; 10 \n 13 \r
detecting: 
  db "Detecting Memory Success !", 10, 13, 0; 10 \n 13 \r

error:
  mov si, .msg
  call print
  hlt; cpu停机
  jmp $
  .msg: db "Loading Error!!!", 10, 13, 0

[bits 32]
protected_mode:
  ; 初始化段寄存器
  mov ax, data_selector
  mov ds, ax
  mov es, ax
  mov fs, ax
  mov gs, ax
  mov ss, ax

  mov esp, 0x10000; 修改栈顶,向低地址增长

  ; 内核地址,向高低地址增长
  mov edi, 0x10000
  mov ecx, 10
  mov bl, 200
  call read_disk

  ; 进入内核
  jmp dword code_selector:0x10000

; 阻塞
jmp $

read_disk:
  ;设置读写扇区数量
  mov dx, 0x1f2;读取扇区数量的端口
  mov al, bl; 获取读取扇区数的参数
  out dx, al; 写入端口

  ; 设置起始扇区,低8位
  inc dx; 0x1F3 设置0-8位
  mov al, cl; 起始扇区的中8位
  out dx, al; 写入前8位

  ; 设置起始扇区,中8位
  inc dx; 0x1F4 设置0-8位
  shr ecx, 8; eax右移8位
  mov al, cl; 起始扇区的高8位
  out dx, al; 写入前8位

  ; 设置起始扇区高8位
  inc dx; 0x1F5 设置0-8位
  shr ecx, 8; eax右移8位
  mov al, ch; 起始扇区的高8位
  out dx, al; 写入前8位

  ; 设置起始扇区高四位,及读取模式
  inc dx; 0x1F6
  shr ecx, 8
  and cl, 0b0000_1111; 将高四位置为0
  mov al, 0b1110_0000; 5,6,7都是1,第6位表示LBA模式,第4位为0表示是主盘
  ; 合并al,cl
  or al, cl
  out dx, al

  ; 输出
  inc dx; 0x1F7
  mov al, 0x20; 表示读硬盘
  out dx, al

  ; 清空ecx,为读取设置参数
  xor ecx, ecx
  mov cl, bl; 得到读写扇区的数量

  ; 读取数据主流程
  ; 先等待数据准备完毕
  ; 然后去读取
  .read:
    push cx; 保存cx,下面的read_to_mem修改了cx
    call .read_waits; 等待数据准备完毕
    call .read_to_mem; 读取一个扇区
    pop cx; 恢复cx
    loop .read
  ret

  ; 等待数据准备完毕
  .read_waits:
    mov dx, 0x1f7
    .read_check: ; 检查数据是否准备完毕
      in al, dx; 将数据读到al寄存器
      ; 做一些延时
      jmp $ + 2
      jmp $ + 2
      jmp $ + 2
      ; 只获取第三位和第七位
      ; 第三位表示数据是否准备完毕
      ; 第七位表示硬盘是否忙
      and al, 0b1000_1000;
      ; 判断数据是否准备完毕
      cmp al, 0b0000_1000;
      ; 没有准备完毕,继续空转cpu,直到准备完毕
      jnz .read_check
    ; 准备完毕,check返回,开始读硬盘
    ret

  ; 将数据读到内存
  .read_to_mem:
    mov dx, 0x1f0; 读数据的端口
    mov cx, 256; 一个扇区256个字
    .read_w:
      in ax, dx
      ; 做一下延时
      jmp $ + 2
      jmp $ + 2
      jmp $ + 2
      ; 将数据读到edi
      mov [edi], ax
      ; 将edi挪动到下一个存放数据的地址
      add edi, 2
      loop .read_w
    ret

; 代码段选择子
code_selector equ (1 << 3)
; 数据段选择子
data_selector equ (2 << 3)

; 内存开始的地址
memory_base equ 0
; 内存界限(4G / 4K) - 1
memory_limit equ ((1024 * 1024 * 1024 * 4) / (1024 * 4)) - 1

; 全局描述符表指针
gdt_ptr:
  ; gdt 界限,长度-1
  dw (gdt_end - gdt_base) - 1
  ; gdt起始位置
  dd  gdt_base
gdt_base:
  dd 0, 0; NULL 描述符

; 定义变量
gdt_mode_h_addr equ  0b_1100_0000 | (memory_limit >> 16) & 0xf

; 代码段
gdt_code:
  dw memory_limit & 0xffff; 段界限的0-15位
  dw memory_base &  0xffff; 基地址的0-16位
  db (memory_base >> 16) & 0xffff; 基地址高8位
  ; 存在 dpl0, S 代码 非依从, 可读, 没有被访问过
  db 0b_1001_1010
  ; 4K 32位,不是64位 段界限的16-19位
  db gdt_mode_h_addr
  db (memory_base >> 24) & 0xff; 基地址24-31位

; 数据段
gdt_data:
  dw memory_limit & 0xffff; 段界限的0-15位
  dw memory_base &  0xffff; 基地址的0-16位
  db (memory_base >> 16) & 0xffff; 基地址高8位
  ; 存在 dpl0, S 数据 向上, 可写, 没有被访问过
  db 0b_1001_0010
  ; 4K 32位,不是64位 段界限的16-19位
  db gdt_mode_h_addr
  db (memory_base >> 24) & 0xff; 基地址24-31位
gdt_end:

addrs_count:
  dw 0
addrs_buffer: