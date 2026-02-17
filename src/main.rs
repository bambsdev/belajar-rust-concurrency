use std::thread::current;

fn main() {
    let binding = current();
    let name_thread = binding.name();
    let name = match name_thread {
        None => "no",
        Some(v) => v,
    };
    println!("This Program running in thread : {}", name);
}

#[cfg(test)]
mod tests {
    use std::{
        cell::RefCell,
        sync::{
            Arc, Barrier, Mutex, Once,
            atomic::{AtomicI32, Ordering},
            mpsc::channel,
        },
        thread::{Builder, current, sleep, spawn},
        time::Duration,
    };

    use tokio::runtime::Runtime;

    // ! Chapter 1 : Membuat thread : spawn()

    #[test]
    fn test_create_thread() {
        spawn(|| {
            for counter in 0..=5 {
                println!("Counter : {}", counter);
                sleep(Duration::from_secs(1));
            }
        });
        println!("Program Finish");
        sleep(Duration::from_secs(7));
    }

    fn calculate() -> i32 {
        let current = current();
        let mut counter = 0;
        for i in 0..=5 {
            match current.name() {
                None => println!("{:?} Counter : {}", current.id(), i),
                Some(name) => println!("{} Counter : {}", name, i),
            }
            sleep(Duration::from_secs(1));
            counter += 1;
        }
        counter
    }

    // ! Chapter 2 : Join thread : method spawn -> join() > join = bloking/menunggu
    #[test]
    fn test_join_thread() {
        let handle = spawn(|| calculate());

        println!("Waiting Handle");

        let result = handle.join();
        match result {
            Ok(counter) => println!("Total counter : {}", counter),
            Err(err) => println!("Error {:?}", err),
        }

        println!("Application Done");
    }

    #[test]
    fn test_sequential() {
        let result1 = calculate();
        let result2 = calculate();
        println!("Result 1 : {}", result1);
        println!("Result 2 : {}", result2);
        println!("Program Finish");
    }

    #[test]
    fn test_parallel() {
        let handle1 = spawn(|| calculate());
        let handle2 = spawn(|| calculate());

        let result1 = handle1.join();
        let result2 = handle2.join();

        match result1 {
            Ok(counter) => println!("Total Counter 1 : {}", counter),
            Err(err) => println!("Error : {:?}", err),
        }

        match result2 {
            Ok(counter) => println!("Total Counter 1 : {}", counter),
            Err(err) => println!("Error : {:?}", err),
        }
    }

    // ! Chapter 3 : move -> memindahkan ownership, menghindari dangling pointer
    #[test]
    fn test_closure() {
        let user = String::from("Ibrohim");
        let closure = move || {
            sleep(Duration::from_secs(2));
            println!("Hallo {}!", user);
        };
        let handle = spawn(closure);
        handle.join().unwrap();

        // println!("{}", user); // Error karena owner sudah dipindahkan
    }
    // ! Chapter 4 : current thread
    // thread bersumber dari spawn tidak punya nama.
    // kalau thread utama ada namanya.

    // ! Chapter 5 : thread factory -> membuat thread sendiri
    // contoh menggunkaan thread factory untuk menambahkan nama custom di thread

    #[test]
    fn test_thread_factory() {
        let factory = Builder::new().name("My Thread".to_string());
        let handle = factory
            .spawn(calculate)
            .expect("Failed to create a new thread"); // .expect() -> pastikan harus berhasil. supaya Result hilang(ok, err)

        let total = handle.join().unwrap();
        println!("{}", total);
    }

    // ! Chapter 5 : channel -> kirim data antar thread

    #[test]
    fn test_channel() {
        let (sender, receiver) = channel::<String>();
        let handle1 = spawn(move || {
            sleep(Duration::from_secs(2));
            sender.send("Hollo From Thread".to_string())
        });
        let handle2 = spawn(move || {
            let message = receiver.recv().unwrap();
            println!("{}", message);
        });
        let _ = handle1.join();
        let _ = handle2.join();
    }

    #[test]
    fn test_channel_queue() {
        let (sender, receiver) = channel::<String>();
        let handle1 = spawn(move || {
            for i in 0..5 {
                sleep(Duration::from_secs(2));
                sender
                    .send(format!("Hello this iterasi at : {}", i))
                    .unwrap();
            }
            sender.send("Exit".to_string())
        });
        let handle2 = spawn(move || {
            loop {
                let message = receiver.recv().unwrap();
                if message == "Exit" {
                    break;
                }
                println!("{}", message);
            }
        });
        let _ = handle1.join();
        let _ = handle2.join();
    }
    #[test]
    fn test_channel_iterator() {
        let (sender, receiver) = channel::<String>();
        let handle1 = spawn(move || {
            for i in 0..5 {
                sleep(Duration::from_secs(2));
                let _ = sender.send(format!("Hello this iterasi at : {}", i));
            }
        });
        let handle2 = spawn(move || {
            for value in receiver.iter() {
                println!("{}", value);
            }
        });
        let _ = handle1.join();
        let _ = handle2.join();
    }
    #[test]
    fn test_channel_multi_sender() {
        let (sender, receiver) = channel::<String>();
        let sender1 = sender.clone();

        let handle1 = spawn(move || {
            for i in 0..5 {
                sleep(Duration::from_secs(2));
                let _ = sender.send(format!("Hello from sender 1 this iterasi at : {}", i));
            }
        });
        let handle2 = spawn(move || {
            for i in 0..5 {
                sleep(Duration::from_secs(2));
                let _ = sender1.send(format!("Hello from sender 2 this iterasi at : {}", i));
            }
        });
        let handle3 = spawn(move || {
            for value in receiver.iter() {
                println!("{}", value);
            }
        });
        let _ = handle1.join();
        let _ = handle2.join();
        let _ = handle3.join();
    }

    // ! Chapter 7 : Malasalah race condition

    static mut COUNTER: i32 = 0;
    #[test]
    fn test_race_condition() {
        let mut handles = vec![];
        for _ in 0..10 {
            let handle = spawn(|| unsafe {
                for _ in 0..10000000 {
                    COUNTER += 1;
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap()
        }
        println!("Counter : {}", unsafe { COUNTER });
    }

    // ! Chapter 8 : Atomic

    #[test]
    fn test_atomic() {
        static COUNTER: AtomicI32 = AtomicI32::new(0);

        let mut handles = vec![];
        for _ in 0..10 {
            let handle = spawn(|| {
                for _ in 0..10000000 {
                    COUNTER.fetch_add(1, Ordering::Relaxed);
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap()
        }
        println!("Counter : {}", COUNTER.load(Ordering::Relaxed));
    }

    // ! Chapter 9 : Atomic Rc / Arc / Atomic Reference counted

    #[test]
    fn test_atomic_reference() {
        let counter = Arc::new(AtomicI32::new(0));

        let mut handles = vec![];
        for _ in 0..10 {
            let counter_clone = Arc::clone(&counter);
            let handle = spawn(move || {
                for _ in 0..10000000 {
                    counter_clone.fetch_add(1, Ordering::Relaxed);
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap()
        }
        println!("Counter : {}", counter.load(Ordering::Relaxed));
    }

    // ! Chapter 10 : Mutex
    #[test]
    fn test_mutex() {
        let counter = Arc::new(Mutex::new(0));

        let mut handles = vec![];
        for _ in 0..10 {
            let counter_clone = Arc::clone(&counter);
            let handle = spawn(move || {
                for _ in 0..10000000 {
                    let mut data = counter_clone.lock().unwrap();
                    *data += 1;
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap()
        }
        println!("Counter : {}", *counter.lock().unwrap());
    }

    // ! Chapter 11 : local Thred -> alur hidup yang mengikuti threadnya
    thread_local! {
        pub static NAME : RefCell<String> = RefCell::new("Default".to_string())
    }
    thread_local! {
        pub static OTHER_NAME : RefCell<String> = RefCell::new("Default".to_string())
    }
    #[test]
    fn test_local_thread() {
        let handle = spawn(|| {
            NAME.with_borrow_mut(|name| {
                *name = "Budi".to_string();
            });

            NAME.with_borrow(|name| {
                println!("{}", name);
            })
        });
        handle.join().unwrap();
        NAME.with_borrow(|name| {
            println!("{}", name);
        })
    }
    // ! Chapter 12 : thread panic -> panic terisolasi di thread tersebut
    #[test]
    fn test_thread_panic() {
        let handle = spawn(|| panic!("oops, someting went wrong"));
        match handle.join() {
            Ok(_) => println!("Thread Finish"),
            Err(_) => println!("Thread Panic"),
        }
    }
    // ! Chapter 13 : barrier -> thread menunggu supaya semua thread siap, setelah itu jalan bareng, seperti balapan lari.
    #[test]
    fn test_barrier() {
        let barrier = Arc::new(Barrier::new(10));
        let mut handles = vec![];
        for i in 0..10 {
            let barrier_clone = Arc::clone(&barrier);
            let handle = spawn(move || {
                println!("Join Game-{}", i);
                barrier_clone.wait();
                println!("Game-{} Start", i);
            });
            handles.push(handle);
        }
    }
    // ! Chapter 13 : Once -> hanya satu thread pertama yang bisa menginisialisasi datanya.
    static mut TOTAL_COUNTER: i32 = 0;
    static TOTAL_INIT: Once = Once::new();

    fn get_total() -> i32 {
        unsafe {
            TOTAL_INIT.call_once(|| {
                TOTAL_COUNTER += 1;
            });
            return TOTAL_COUNTER;
        }
    }
    #[test]
    fn test_once() {
        let mut handles = vec![];
        for _ in 0..10 {
            let handle = spawn(|| {
                let total = get_total();
                println!("Total : {}", total);
            });
            handles.push(handle);
        }
        for handle in handles {
            handle.join().unwrap()
        }
    }
    // ! Chapter 14 : async await. menggunakan tokio sebagai runtime/executor nya.
    async fn get_async_data() -> String {
        tokio::time::sleep(Duration::from_secs(2)).await;
        println!("Hello from async");
        return "Hello from async".to_string();
    }

    #[tokio::test]
    async fn test_async() {
        let _function = get_async_data();
        println!("Finish call async");
        let data = get_async_data().await;
        println!("{}", data);
    }

    // ! Chapter 14 : tast -> implementasi cuncarrency. pakai tokio::spawn()
    async fn get_database(wait: u64) -> String {
        println!("{:?} get database data", current().id());
        tokio::time::sleep(Duration::from_secs(wait)).await;
        println!("{:?} hello from database", current().id());
        "Hello From Database".to_string()
    }
    #[tokio::test]
    async fn test_cuncarrency() {
        let mut handles = vec![];
        for i in 0..5 {
            let handle = tokio::spawn(get_database(i));
            handles.push(handle);
        }
        for handle in handles {
            let data = handle.await.unwrap();
            println!("Response : {}", data);
        }
    }

    // ! Chapter 14 : tokio runtime -> custom runtimenya tokio biar lebih gahar / implementasi thread + task
    async fn run_concurrency_and_parallel(runtime: Arc<Runtime>) {
        let mut handles = vec![];
        for i in 0..5 {
            let handle = runtime.spawn(get_database(i));
            handles.push(handle);
        }
        for handle in handles {
            let data = handle.await.unwrap();
            println!("Response : {}", data);
        }
    }

    #[test]
    fn test_tokio_runtime() {
        let runtime = Arc::new(
            tokio::runtime::Builder::new_multi_thread()
                .worker_threads(10)
                .enable_time()
                .build()
                .unwrap(),
        );
        runtime.block_on(run_concurrency_and_parallel(Arc::clone(&runtime)))
    }
}
