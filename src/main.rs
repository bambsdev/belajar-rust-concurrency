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
        sync::{
            atomic::{AtomicI32, Ordering},
            mpsc::channel,
        },
        thread::{Builder, current, sleep, spawn},
        time::Duration,
    };
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

        // println!("{}", name); // Error karena owner sudah dipindahkan
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

    static OTHER_COUNTER: AtomicI32 = AtomicI32::new(0);
    #[test]
    fn test_atomic() {
        let mut handles = vec![];
        for _ in 0..10 {
            let handle = spawn(|| {
                for _ in 0..10000000 {
                    OTHER_COUNTER.fetch_add(1, Ordering::Relaxed);
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap()
        }
        println!("Counter : {}", OTHER_COUNTER.load(Ordering::Relaxed));
    }
}
