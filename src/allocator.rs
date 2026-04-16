// 禁用所有警告，只保留错误
#![allow(warnings)]

use core::alloc::{GlobalAlloc, Layout};
use core::mem::size_of;
use core::ptr::null_mut;

static mut HEAP: [u8; 1024 * 1024] = [0; 1024 * 1024]; // 1 MiB 堆空间
static mut HEAP_INITIALIZED: bool = false;
static mut HEAP_START: usize = 0x0;
static mut HEAP_PTR: usize = 0x0;
static HEAP_SIZE: usize = 1024 * 1024; // 1 MiB

// #[unsafe(no_mangle)]
// pub extern "C" fn rust_eh_personality() {}

#[cfg(target_arch = "x86_64")]
#[unsafe(no_mangle)]
pub extern "C" fn rust_eh_personality() {}

#[cfg(target_arch = "arm")]
#[unsafe(no_mangle)]
pub extern "C" fn rust_eh_personality() {}

#[unsafe(no_mangle)]
pub extern "C" fn memset(s: *mut u8, c: i32, n: usize) -> *mut u8 {
    unsafe {
        for i in 0..n {
            *s.add(i) = c as u8;
        }
    }
    s
}

#[unsafe(no_mangle)]
pub extern "C" fn memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    unsafe {
        for i in 0..n {
            *dest.add(i) = *src.add(i);
        }
    }
    dest
}

#[unsafe(no_mangle)]
pub extern "C" fn memmove(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    unsafe {
        if dest < src as *mut u8 {
            // 正向复制
            for i in 0..n {
                *dest.add(i) = *src.add(i);
            }
        } else {
            // 反向复制（避免重叠覆盖）
            for i in (0..n).rev() {
                *dest.add(i) = *src.add(i);
            }
        }
    }
    dest
}

/**
 * 将地址向上对齐到指定的对齐值
 */
fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

/**
 * 计算包含头部信息的总大小
 */
fn size_of_with_header<T>() -> usize {
    size_of::<BlockHeader>() + size_of::<T>()
}

fn heap_init() {
    unsafe {
        if !HEAP_INITIALIZED {
            HEAP_START = (&raw const HEAP) as *const u8 as usize;
            HEAP_PTR = HEAP_START;
            HEAP_INITIALIZED = true;
        }
    }
}

/**
 * 从堆中分配内存，返回指向块头的指针
 */
unsafe fn alloc_from_heap(size: usize, align: usize) -> *mut BlockHeader {
    heap_init();
    let header_size = size_of::<BlockHeader>();
    // 确保 user_ptr 对齐数至少为 align_of::<BlockHeader>()，从而保证 block_ptr 满足 BlockHeader 对齐要求
    let effective_align = align.max(core::mem::align_of::<BlockHeader>());
    // 用户数据地址需要对齐，所以先对 (HEAP_PTR + header_size) 向上对齐
    let user_ptr = align_up(HEAP_PTR + header_size, effective_align);
    let block_ptr = user_ptr - header_size; // BlockHeader 紧在用户数据之前
    let total_size = user_ptr - HEAP_PTR + size;
    // 检查是否有足够的堆空间
    if user_ptr + size > HEAP_START + HEAP_SIZE {
        return null_mut();
    }
    let block_header = block_ptr as *mut BlockHeader;
    (*block_header).size = size;
    (*block_header).in_use = true;
    HEAP_PTR = user_ptr + size;
    block_header
}

#[repr(C)]
struct BlockHeader {
    size: usize,
    in_use: bool,
}

impl BlockHeader {
    unsafe fn from_user_ptr(ptr: *mut u8) -> *mut BlockHeader {
        // 先在字节级减去 header 大小，再转换指针类型，避免不对齐指针上做 offset 运算
        ptr.sub(size_of::<BlockHeader>()) as *mut BlockHeader
    }

    unsafe fn user_ptr(&self) -> *mut u8 {
        (self as *const BlockHeader as *mut u8).add(size_of::<BlockHeader>())
    }

    unsafe fn next_block(&self) -> *mut BlockHeader {
        (self as *const BlockHeader as *mut u8).add(self.size) as *mut BlockHeader
    }
}

// 空闲块的链表节点
#[repr(C)]
struct FreeBlockNode {
    size: usize,              // 块大小
    next: *mut FreeBlockNode, // 指向下一个空闲块
}

struct FreeList {
    head: *mut FreeBlockNode,
}

impl FreeList {
    fn new() -> Self {
        FreeList {
            head: core::ptr::null_mut(),
        }
    }
}

impl FreeList {
    unsafe fn push_block_node(this: *mut Self, block: *mut FreeBlockNode, size: usize) {
        (*block).size = size;
        (*block).next = (*this).head;
        (*this).head = block;
    }

    unsafe fn push(this: *mut Self, block: *mut BlockHeader) {
        let free_block = block as *mut FreeBlockNode;
        (*free_block).next = core::ptr::null_mut();
        Self::push_block_node(this, free_block, (*block).size);
    }

    unsafe fn find_and_remove(
        this: *mut Self,
        required_size: usize,
    ) -> Option<(*mut FreeBlockNode, usize)> {
        let mut current = (*this).head;
        let mut prev: *mut *mut FreeBlockNode = &mut (*this).head;

        while !current.is_null() {
            if (*current).size >= required_size {
                // 从链表移除
                *prev = (*current).next;
                return Some((current, (*current).size));
            }
            prev = &mut (*current).next;
            current = (*current).next;
        }
        None
    }
}

static mut FREE_LIST: FreeList = FreeList {
    head: core::ptr::null_mut(),
};

pub struct SimpleAllocator;

impl SimpleAllocator {
    pub const fn new() -> Self {
        SimpleAllocator
    }
}

unsafe impl GlobalAlloc for SimpleAllocator {
    /**
     * 分配内存的核心函数，首先尝试从空闲链表中分配，如果没有合适的块，则从堆中分配
     */
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let align = layout.align();
        let size = layout.size();
        // 首先尝试从空闲链表中分配
        if let Some((free_block, block_size)) = FreeList::find_and_remove(
            &raw mut FREE_LIST as *mut FreeList,
            size + size_of::<BlockHeader>(),
        ) {
            let block_header = free_block as *mut BlockHeader;
            (*block_header).size = block_size;
            (*block_header).in_use = true;
            return (*block_header).user_ptr();
        }
        // 如果空闲链表中没有合适的块，则从堆中分配
        let block_header = alloc_from_heap(size, align);
        if block_header.is_null() {
            null_mut() // 堆空间不足
        } else {
            (*block_header).user_ptr()
        }
    }

    /**
     * 释放内存的核心函数，将释放的块加入空闲链表
     */
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let block_header: *mut BlockHeader = BlockHeader::from_user_ptr(ptr);
        // 将释放的块加入空闲链表
        FreeList::push(&raw mut FREE_LIST as *mut FreeList, block_header);
    }

    /**
     * 分配零初始化的内存，直接调用 alloc 分配内存后将其清零
     */
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let user_ptr = self.alloc(layout);
        if !user_ptr.is_null() {
            core::ptr::write_bytes(user_ptr, 0, layout.size());
        }
        user_ptr
    }

    /**
     * 重新分配内存的核心函数，新分配一个块并复制旧数据，如果新大小小于等于当前块大小，则直接返回原指针
     */
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let block_header: *mut BlockHeader = BlockHeader::from_user_ptr(ptr);

        if new_size <= (*block_header).size {
            // 如果新大小小于等于当前块大小，直接返回原指针
            return ptr;
        }
        // 否则，分配一个新的块并复制旧数据
        let new_layout = Layout::from_size_align(new_size, layout.align()).unwrap();
        let new_ptr = self.alloc(new_layout);
        if !new_ptr.is_null() {
            core::ptr::copy_nonoverlapping(ptr, new_ptr, layout.size());
            self.dealloc(ptr, layout);
        }
        new_ptr
    }
}
