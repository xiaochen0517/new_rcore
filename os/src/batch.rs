use core::{
    cell::RefMut,
    ops::DerefMut,
    slice::{from_raw_parts, from_raw_parts_mut},
};

use lazy_static::lazy_static;
use log::{debug, error, info};

use crate::{sbi::shutdown, sync::up::UpSafeCell, trap::context::TrapContext};

// 应用加载到的目标地址
const APP_BASE_ADDRESS: usize = 0x80_400_000;
// 应用最大大小限制
const APP_MAX_SIZE: usize = 0x20_000; // 1 MiB
// 表示最大支持的用户程序数量
const MAX_APP_NUM: usize = 16;

// AppManager 结构体，管理用户程序的信息
pub struct AppManager {
    num_app: usize,
    current_app: usize,
    app_start: [usize; MAX_APP_NUM + 1],
}

// 初始化 AppManager，读取 APP 数量和起始地址列表
lazy_static! {
    static ref APP_MANAGER: UpSafeCell<AppManager> = unsafe {
        UpSafeCell::new({
            unsafe extern "C" {
                fn _num_app();
            }
            // 获取 APP 数量指针
            let num_app_ptr = _num_app as *const usize;
            // 读取 APP 数量
            let num_app = num_app_ptr.read_volatile();
            // 读取 APP 起始地址列表
            let mut app_start: [usize; MAX_APP_NUM + 1] = [0; MAX_APP_NUM + 1];
            // 从 num_app_ptr 后面紧跟着的内存位置开始，读取 num_app + 1 个 usize 作为 APP 的起始地址
            let app_start_raw: &[usize] =
                core::slice::from_raw_parts(num_app_ptr.add(1), num_app + 1);
            app_start[..=num_app].copy_from_slice(app_start_raw);
            AppManager {
                num_app,
                current_app: 0,
                app_start,
            }
        })
    };
}

impl AppManager {
    // 输出当前 APP 的信息
    pub fn print_app_info(&self) {
        info!("[batch] Found {} app(s)", self.num_app);
        for i in 0..self.num_app {
            info!(
                "[batch] App {} entry point: {:#x} to {:#x}",
                i,
                self.app_start[i],
                self.app_start[i + 1]
            );
        }
    }

    pub fn move_to_next_app(&mut self) {
        self.current_app += 1;
    }

    // 加载 APP
    pub fn load_app(&mut self, app_id: usize) {
        if app_id >= self.num_app {
            error!(
                "[batch] Invalid app_id: {}; Max app_id: {}",
                app_id,
                self.num_app - 1
            );
            shutdown(false);
        }
        info!("[batch] Loading app {}...", app_id);
        let app_size = self.app_start[app_id + 1] - self.app_start[app_id];
        if app_size > APP_MAX_SIZE {
            error!(
                "[batch] App {} size {} exceeds maximum allowed size {}",
                app_id, app_size, APP_MAX_SIZE
            );
            shutdown(false);
        }
        unsafe {
            // 首先需要清理当前目标加载位置的内存，确保没有残留数据
            from_raw_parts_mut(APP_BASE_ADDRESS as *mut u8, APP_MAX_SIZE).fill(0);
            // 将 APP 的内容从其存储位置复制到目标加载地址
            let app_src = from_raw_parts(self.app_start[app_id] as *const u8, app_size);
            debug!(
                "[batch] read app size: {} bytes from {:#x}",
                app_size, self.app_start[app_id]
            );
            let app_dst = from_raw_parts_mut(APP_BASE_ADDRESS as *mut u8, app_src.len());
            app_dst.copy_from_slice(app_src);
            debug!(
                "[batch] App {} loaded to {:#x} (size: {} bytes)",
                app_id, APP_BASE_ADDRESS, app_size
            );
            // 刷新指令缓存，确保 CPU 能正确执行新加载的代码
            core::arch::asm!("fence.i");
        }
    }
}

pub fn init() {
    APP_MANAGER.exclusive_access().print_app_info();
}

const USER_STACK_SIZE: usize = 4096 * 2;
const KERNEL_STACK_SIZE: usize = 4096 * 2;

#[repr(align(4096))]
struct KernelStack {
    data: [u8; KERNEL_STACK_SIZE],
}

#[repr(align(4096))]
struct UserStack {
    data: [u8; USER_STACK_SIZE],
}

static KERNEL_STACK: KernelStack = KernelStack {
    data: [0; KERNEL_STACK_SIZE],
};
static USER_STACK: UserStack = UserStack {
    data: [0; USER_STACK_SIZE],
};

impl KernelStack {
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + KERNEL_STACK_SIZE
    }

    pub fn push_context(&self, cx: TrapContext) -> &'static mut TrapContext {
        let cx_ptr = (self.get_sp() - size_of::<TrapContext>()) as *mut TrapContext;
        unsafe {
            *cx_ptr = cx;
            cx_ptr.as_mut().unwrap()
        }
    }
}

impl UserStack {
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + USER_STACK_SIZE
    }
}

pub fn run_next_app() -> ! {
    let mut app_manager = APP_MANAGER.exclusive_access();
    let current_app = app_manager.current_app;
    app_manager.load_app(current_app);
    app_manager.move_to_next_app();
    drop(app_manager);
    info!("[batch] Running app {}...", current_app);
    unsafe extern "C" {
        fn __restore(cx_addr: usize);
    }
    unsafe {
        __restore(KERNEL_STACK.push_context(TrapContext::app_init_context(
            APP_BASE_ADDRESS,
            USER_STACK.get_sp(),
        )) as *const _ as usize);
    }
    panic!("Unreachable code after __restore!");
}
