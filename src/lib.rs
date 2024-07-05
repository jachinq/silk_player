// #![allow(non_snake_case)]

use std::sync::{mpsc, Arc, Mutex};
use std::thread;

type Job = Box<dyn FnOnce() + Send + 'static>;

// 定义一个消息体枚举类，来判断当前线程是执行工作，还是执行停机
enum Message {
    NewJob(Job), // 工作
    Terminate,   // 停机
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}
impl Default for ThreadPool {
    fn default() -> Self {
        Self::new(1)
    }
}

impl ThreadPool {
    /// 创建线程池
    ///
    /// 线程池中线程的数量
    ///
    /// # Panics
    ///
    /// `new` 函数会在 size 为 0 时触发 panic
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);
        let mut workers = Vec::with_capacity(size); // 根据数量来创建指定大小的集合

        // rust 中的通道，是多生产者，单消费者的
        let (sender, receiver) = mpsc::channel();
        // 如果想共享消费者，则必须使用 Arc 和 Mutex 来进行共享。
        let receiver = Arc::new(Mutex::new(receiver));

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.send(Message::NewJob(job)).unwrap();
        // f();
    }
}

struct Worker {
    _id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || {
            loop {
                // recv 会阻塞当前线程，直到它从通道中取出信号，处理完当前工作后，它会循环进入下一次 recv 阻塞等待
                let message = receiver.lock().unwrap().recv().unwrap();
                match message {
                    // 如果是有工作，则执行工作
                    Message::NewJob(job) => {
                        // println!("worker {} get a job; executing.", id);
                        job();
                    }
                    // 如果是需要停机，则结束死循环
                    Message::Terminate => {
                        // println!("worker {} was told to terminate.", id);
                        break;
                    }
                }
            }
        });
        let thread = Some(thread);
        Worker { _id: id, thread }
    }
}

// 优雅停机
impl Drop for ThreadPool {
    // 为线程池实现 drop 方法。
    // 我们希望实现两点：
    // 1.给线程池中的所有线程发送停机信号
    // 2.当线程池离开作用域时，我们希望还在工作的线程，能通过 join 阻塞来完成它的工作，然后再删除整个线程池
    fn drop(&mut self) {
        // println!("Sending terminate message to all worker.");
        for _ in &mut self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }
        // println!("Shutting all worker.");

        for worker in &mut self.workers {
            // println!("Shutting down worker {}", worker.id);
            if let Some(thread) = worker.thread.take() {
                // join 要求持有所有权时才能使用，我们可以将 Worker 的 thread 用 Option 包裹起来，
                // 通过 take 得到所有权后，留下 None，而对于 thread 为 None 的就意味着已经被清理过了
                thread.join().unwrap()
            }
        }
    }
}
