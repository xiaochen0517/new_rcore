.altmacro
.macro SAVE_GP n
    # 将 x<n> 保存到 TrapContext.x[n]，偏移为 sp + n*8。
    sd x\n, \n*8(sp)
.endm
.macro LOAD_GP n
    # 从 TrapContext.x[n] 恢复 x<n>，偏移为 sp + n*8。
    ld x\n, \n*8(sp)
.endm
    .section .text
    .globl __alltraps
    .globl __restore
    .align 2
__alltraps:
    # 从用户态陷入后进入此处，交换 sp 与 sscratch：
    # - sp 变为内核预先准备好的内核栈栈顶
    # - sscratch 暂存用户栈指针，供后续恢复
    csrrw sp, sscratch, sp
    # 在内核栈上为 TrapContext 预留空间：
    # 32 个通用寄存器槽位 + sstatus + sepc = 34 个 u64
    addi sp, sp, -34*8
    # 将通用寄存器保存到 TrapContext。
    sd x1, 1*8(sp)
    # x2(sp) 对应用户栈指针，稍后从 sscratch 中取出再保存。
    sd x3, 3*8(sp)
    # x4(tp) 在本实验的用户程序中未使用，跳过。
    # 用循环保存 x5..x31。
    .set n, 5
    .rept 27
        SAVE_GP %n
        .set n, n+1
    .endr
    # 保存与陷阱返回相关的 CSR 到 TrapContext：
    # - sstatus：陷入时的特权/状态位
    # - sepc：返回用户态时的 PC
    csrr t0, sstatus
    csrr t1, sepc
    sd t0, 32*8(sp)
    sd t1, 33*8(sp)
    # 将 sscratch 中保存的原始用户 sp 写入 TrapContext.x[2]。
    csrr t2, sscratch
    sd t2, 2*8(sp)
    # 以 a0=&mut TrapContext 的形式调用 Rust 的 trap_handler。
    mv a0, sp
    call trap_handler

__restore:
    # 两种场景会进入这里：
    # 1) 应用第一次启动
    # 2) 处理 trap 后返回用户态
    # 约定 a0 为 TrapContext 指针。
    mv sp, a0
    # 从 TrapContext 恢复 CSR 与用户 sp 快照。
    ld t0, 32*8(sp)
    ld t1, 33*8(sp)
    ld t2, 2*8(sp)
    csrw sstatus, t0
    csrw sepc, t1
    csrw sscratch, t2
    # 恢复通用寄存器（不含 x2(sp)/x4(tp)）。
    ld x1, 1*8(sp)
    ld x3, 3*8(sp)
    .set n, 5
    .rept 27
        LOAD_GP %n
        .set n, n+1
    .endr
    # 释放内核栈上的 TrapContext 栈帧。
    addi sp, sp, 34*8
    # 再次交换：sp 回到用户栈，sscratch 持有内核栈指针。
    csrrw sp, sscratch, sp
    # 按 sstatus/sepc 指定的特权级与 PC 执行 sret 返回。
    sret
