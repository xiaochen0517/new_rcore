//! 协程底层原理示例（无外部依赖）
//!
//! 核心概念：
//!   - 协程（Coroutine）：可以在执行中途"挂起"，稍后从断点"恢复"的函数
//!   - 状态机（State Machine）：Rust 编译器将 async fn 转换为状态机来实现协程
//!   - 执行器（Executor）：调度并驱动多个协程运行的调度器
//!   - Waker：当某个协程"可以继续运行"时，用来通知执行器的机制
//!
//! 运行后观察输出，可以看到 3 个协程在单线程上交替推进的过程。

use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

// ─── Part 1: Waker ────────────────────────────────────────────────────────────
//
// Waker 是协程与执行器之间的"唤醒通知"接口。
// 当 IO 就绪、定时器到期等外部事件发生时，通过 Waker.wake() 通知执行器
// 重新 poll 对应的协程（避免执行器盲目轮询浪费 CPU）。
//
// 本示例使用最简单的"轮询"策略，不依赖事件通知，
// 因此 Waker 是空操作（noop）。真实运行时（如 tokio）的 Waker
// 会在 wake() 中把任务重新入队，实现事件驱动调度。

fn noop_waker() -> Waker {
    // RawWaker 需要一张函数表（vtable），定义 clone/wake/drop 的行为
    const VTABLE: RawWakerVTable = RawWakerVTable::new(
        |_| RawWaker::new(std::ptr::null(), &VTABLE), // clone：返回新的空 waker
        |_| {},                                       // wake：什么都不做
        |_| {},                                       // wake_by_ref：什么都不做
        |_| {},                                       // drop：什么都不做
    );
    // SAFETY: vtable 中所有函数均是合法的空操作，data 指针也不会被解引用
    unsafe { Waker::from_raw(RawWaker::new(std::ptr::null(), &VTABLE)) }
}

// ─── Part 2: 协程 = 状态机 ────────────────────────────────────────────────────
//
// async fn 的本质：编译器把每个 .await 点当作"暂停点"，
// 将整个函数切分为若干阶段，生成一个实现了 Future trait 的状态机结构体。
//
// 下面手动模拟如下"伪代码"被编译后的等价产物：
//
//   coroutine task(id):
//       print("阶段1：开始")
//       yield          ← 主动让出 CPU，暂停在此
//       print("阶段2：继续")
//       yield          ← 再次让出 CPU
//       print("阶段3：完成")
//
// 每个 yield 对应状态机的一次状态跳转。
// poll() 每次被调用，状态机从当前阶段继续运行到下一个 yield 点或结束。

struct Task {
    id: u32,
    /// 当前所处阶段：0 = 初始，1 = 第一次让出后，2 = 第二次让出后
    phase: u32,
}

impl Task {
    fn new(id: u32) -> Self {
        Task { id, phase: 0 }
    }
}

impl Future for Task {
    type Output = ();

    /// poll 是状态机的"驱动函数"，每次调用推进一步：
    ///   - Poll::Pending   → "我需要暂停，请稍后再来驱动我"
    ///   - Poll::Ready(()) → "我已执行完毕"
    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<()> {
        match self.phase {
            0 => {
                println!("    [协程 {}] 阶段 1：开始执行", self.id);
                self.phase = 1;
                Poll::Pending // ← yield：主动让出，等待下次 poll
            }
            1 => {
                println!("    [协程 {}] 阶段 2：从第一个暂停点恢复", self.id);
                self.phase = 2;
                Poll::Pending // ← yield：再次让出
            }
            2 => {
                println!("    [协程 {}] 阶段 3：从第二个暂停点恢复，完成！", self.id);
                Poll::Ready(()) // ← 执行完毕，状态机终止
            }
            _ => panic!("协程已完成，不应再被 poll"),
        }
    }
}

// ─── Part 3: 执行器 = 调度器 ──────────────────────────────────────────────────
//
// 执行器负责调度所有协程，本示例实现最简单的"轮询式单线程执行器"：
//   1. 维护一个就绪队列（ready queue）
//   2. 每轮依次 poll 队列中的每个协程
//   3. 返回 Ready 的协程已完成，丢弃；返回 Pending 的放入下一轮
//   4. 重复直到队列为空
//
// 这就是"协作式调度"（Cooperative Scheduling）：
//   协程自己决定何时让出 CPU（通过返回 Pending），
//   执行器不会强制打断正在运行的协程。
//   这与操作系统的"抢占式调度"（Preemptive Scheduling）相对。

struct Executor {
    queue: VecDeque<Pin<Box<dyn Future<Output = ()>>>>,
}

impl Executor {
    fn new() -> Self {
        Executor {
            queue: VecDeque::new(),
        }
    }

    /// 向执行器提交一个协程（Future）
    fn spawn(&mut self, task: impl Future<Output = ()> + 'static) {
        // Box::pin：将 Future 分配到堆上并"固定"（Pin）其内存地址。
        // 固定是必要的：Future 状态机内部可能含有自引用指针，
        // 一旦移动内存地址就会导致悬垂指针，Pin 阻止了这种移动。
        self.queue.push_back(Box::pin(task));
    }

    /// 运行所有协程直到全部完成
    fn run(&mut self) {
        let waker = noop_waker();
        let mut cx = Context::from_waker(&waker);
        let mut round = 0;

        while !self.queue.is_empty() {
            round += 1;
            println!(
                "\n  ┌─ 第 {} 轮调度（就绪协程数：{}）",
                round,
                self.queue.len()
            );

            let mut pending = VecDeque::new();

            // 本轮依次 poll 每个协程
            while let Some(mut fut) = self.queue.pop_front() {
                match fut.as_mut().poll(&mut cx) {
                    Poll::Ready(()) => {
                        // 协程已完成，直接丢弃
                    }
                    Poll::Pending => {
                        // 协程主动让出，放入下一轮队列
                        pending.push_back(fut);
                    }
                }
            }

            self.queue = pending;
            println!("  └─ 本轮结束");
        }
    }
}

// ─── 主程序 ───────────────────────────────────────────────────────────────────

fn main() {
    println!("=== 协程调度演示 ===");
    println!("向执行器提交 3 个协程，观察它们在单线程上如何交替执行：");

    let mut executor = Executor::new();

    // 同时提交 3 个协程，它们将被协作式地交替调度
    executor.spawn(Task::new(1));
    executor.spawn(Task::new(2));
    executor.spawn(Task::new(3));

    executor.run();

    println!("\n结论：");
    println!("  - 3 个协程共享 1 个线程，每轮各推进一个阶段后主动让出 CPU");
    println!("  - 协程通过返回 Poll::Pending 实现 yield，执行器负责再次调度");
    println!("  - 这就是协作式多任务的本质，也是 Rust async/await 的底层机制");
}
