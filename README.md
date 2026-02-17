# Belajar Rust Concurrency - Ringkasan Pembelajaran

Dokumentasi lengkap pembelajaran mengenai concurrent programming di Rust, mencakup threading, synchronization primitives, dan async/await dengan Tokio.

## 📋 Daftar Isi

1. [Thread Dasar](#thread-dasar)
2. [Thread Synchronization](#thread-synchronization)
3. [Inter-Thread Communication](#inter-thread-communication)
4. [Race Conditions & Solutions](#race-conditions--solutions)
5. [Advanced Threading](#advanced-threading)
6. [Async/Await dengan Tokio](#asyncawait-dengan-tokio)

---

## Thread Dasar

### 1. Membuat Thread dengan `spawn()`

```rust
spawn(|| {
    println!("Hello from thread");
});
```

**Penjelasan:** `spawn()` membuat thread baru yang berjalan concurrent dengan main thread.

### 2. Joining Thread

```rust
let handle = spawn(|| calculate());
let result = handle.join().unwrap(); // Menunggu thread selesai
```

**Penjelasan:** `join()` adalah blocking operation yang menunggu thread selesai executing dan mengembalikan hasilnya.

### 3. Sequential vs Parallel

- **Sequential:** Eksekusi satu function setelah function lain selesai → Total waktu = sum semua waktu
- **Parallel:** Eksekusi multiple functions bersamaan → Total waktu = waktu function yang paling lama

### 4. Ownership dengan `move` Closure

```rust
let user = String::from("Ibrohim");
let closure = move || println!("Hello {}", user);
spawn(closure);
```

**Penjelasan:** `move` memindahkan ownership ke closure, mencegah dangling pointer. Setelah ini, `user` tidak bisa diakses di outer scope.

---

## Thread Synchronization

### 5. Thread Factory dengan `Builder`

```rust
let handle = Builder::new()
    .name("My Thread".to_string())
    .spawn(calculate)
    .expect("Failed to create thread");
```

**Penjelasan:** Builder pattern untuk membuat thread dengan konfigurasi custom (misalnya: nama thread).

### 6. Current Thread

```rust
let binding = current();
let name_thread = binding.name();
```

**Penjelasan:** Mengakses informasi thread saat ini (ID, nama, dll).

---

## Inter-Thread Communication

### 7. Channel - Kirim Data Antar Thread

```rust
let (sender, receiver) = channel::<String>();
spawn(move || sender.send("Hello".to_string()));
spawn(move || println!("{}", receiver.recv().unwrap()));
```

**Penjelasan:** Channel memungkinkan komunikasi aman antar thread. Sender mengirim data, receiver menerima.

### 8. Channel dengan Queue

```rust
for i in 0..5 {
    sender.send(format!("Message {}", i)).unwrap();
}
sender.send("Exit".to_string()); // Signal untuk berhenti
```

**Penjelasan:** Menggunakan channel untuk mengirim multiple messages secara berurutan.

### 9. Channel Iterator

```rust
for value in receiver.iter() {
    println!("{}", value);
}
```

**Penjelasan:** Mengiterasi values dari channel sampai sender di-drop.

### 10. Multi-Sender Channel

```rust
let sender1 = sender.clone();
// Kedua sender bisa mengirim ke receiver yang sama
```

**Penjelasan:** Cloning sender memungkinkan multiple threads mengirim ke receiver tunggal.

---

## Race Conditions & Solutions

### 11. Race Condition Problem

```rust
static mut COUNTER: i32 = 0;
// Multiple threads mengakses COUNTER tanpa synchronization
// Hasil tidak konsisten ❌
```

**Problem:** Tanpa synchronization, multiple threads bisa mengubah variable secara bersamaan, menyebabkan data corruption.

### 12. Atomic - Solusi Sederhana

```rust
static COUNTER: AtomicI32 = AtomicI32::new(0);
COUNTER.fetch_add(1, Ordering::Relaxed);
```

**Penjelasan:** Atomic operations menjamin operasi akan selesai tanpa interruption dari thread lain.

### 13. Arc (Atomic Reference Counted)

```rust
let counter = Arc::new(AtomicI32::new(0));
let counter_clone = Arc::clone(&counter);
spawn(move || {
    counter_clone.fetch_add(1, Ordering::Relaxed);
});
```

**Penjelasan:** Arc memungkinkan multiple threads memiliki ownership dari data yang sama secara aman.

### 14. Mutex - Locking Mechanism

```rust
let counter = Arc::new(Mutex::new(0));
let mut data = counter_clone.lock().unwrap();
*data += 1;
```

**Penjelasan:** Mutex memastikan hanya satu thread yang bisa access data pada waktu yang sama. `lock()` adalah blocking sampai lock acquired.

---

## Advanced Threading

### 15. Thread Local Storage

```rust
thread_local! {
    pub static NAME: RefCell<String> = RefCell::new("Default".to_string())
}

NAME.with_borrow_mut(|name| {
    *name = "Budi".to_string();
});
```

**Penjelasan:** Setiap thread punya copy sendiri dari data. Berguna untuk caching per-thread atau state management.

### 16. Thread Panic - Isolated Error Handling

```rust
let handle = spawn(|| panic!("oops"));
match handle.join() {
    Ok(_) => println!("Thread Finish"),
    Err(_) => println!("Thread Panic"),
}
```

**Penjelasan:** Panic di satu thread tidak crash program keseluruhan. Error terisolasi dan bisa di-handle.

### 17. Barrier - Synchronization Point

```rust
let barrier = Arc::new(Barrier::new(10)); // Tunggu 10 threads
for i in 0..10 {
    let barrier_clone = Arc::clone(&barrier);
    spawn(move || {
        println!("Ready {}", i);
        barrier_clone.wait(); // Semuathread tunggu di sini
        println!("Go {}", i);
    });
}
```

**Penjelasan:** Barrier mengumpulkan threads di synchronization point. Semua threads lanjut bersamaan setelah semua tiba.

### 18. Once - Initialize Once

```rust
static TOTAL_INIT: Once = Once::new();
TOTAL_INIT.call_once(|| {
    // Kode ini hanya execute sekali, walaupun dipanggil dari multiple threads
    initialize_data();
});
```

**Penjelasan:** Memastikan initialization code berjalan tepat sekali, bahkan dengan concurrent calls.

---

## Async/Await dengan Tokio

### 19. Basic Async Function

```rust
async fn get_async_data() -> String {
    tokio::time::sleep(Duration::from_secs(2)).await;
    "Hello".to_string()
}

#[tokio::test]
async fn test_async() {
    let data = get_async_data().await;
    println!("{}", data);
}
```

**Penjelasan:** Async functions return futures yang bisa di-await. `.await` men-suspend function sampai future complete.

### 20. Task dengan `tokio::spawn()`

```rust
async fn get_database(wait: u64) -> String {
    tokio::time::sleep(Duration::from_secs(wait)).await;
    "Data".to_string()
}

#[tokio::test]
async fn test_concurrency() {
    let mut handles = vec![];
    for i in 0..5 {
        let handle = tokio::spawn(get_database(i));
        handles.push(handle);
    }
    for handle in handles {
        let data = handle.await.unwrap();
    }
}
```

**Penjelasan:** `tokio::spawn()` membuat task (lightweight thread) yang concurrent. Lebih efisien daripada OS threads untuk I/O.

### 21. Custom Tokio Runtime

```rust
let runtime = Arc::new(
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(10)
        .enable_time()
        .build()
        .unwrap(),
);
runtime.block_on(run_concurrency_and_parallel(Arc::clone(&runtime)))
```

**Penjelasan:** Membuat custom runtime dengan kontrol penuh: jumlah worker threads, features yang diaktifkan, dll.

---

## 🎯 Key Takeaways

| Concept           | Use Case                      | Thread-safe? |
| ----------------- | ----------------------------- | ------------ |
| **spawn()**       | Create new threads            | ✓            |
| **join()**        | Wait for thread completion    | ✓            |
| **move closure**  | Transfer ownership            | ✓            |
| **Channel**       | Inter-thread communication    | ✓            |
| **Atomic**        | Simple counter access         | ✓            |
| **Arc**           | Shared ownership              | ✓            |
| **Mutex**         | Exclusive access to data      | ✓            |
| **Thread Local**  | Per-thread storage            | ✓            |
| **Barrier**       | Synchronize multiple threads  | ✓            |
| **Once**          | Initialize once safely        | ✓            |
| **Async/Await**   | Non-blocking I/O operations   | ✓            |
| **Tokio Runtime** | Multi-threaded async executor | ✓            |

---

## 📚 Rust Concurrency Primitives Summary

### Thread-based Concurrency

- **Best for:** CPU-intensive tasks, parallel computation
- **Trade-off:** Lebih heavy, tapi deterministic

### Async Concurrency

- **Best for:** I/O-bound tasks, handling many concurrent connections
- **Trade-off:** Lebih lightweight, tapi lebih complex mental model

### Synchronization Options

1. **No synchronization** → DANGER: Race conditions
2. **Atomic** → Untuk simple operations (counters)
3. **Mutex** → Untuk complex data structures
4. **RwLock** → Untuk many readers, few writers
5. **Channel** → Untuk message passing
6. **Barrier/Once** → Untuk coordination

---

## 🔗 Referensi

- [Rust Book - Concurrency](https://doc.rust-lang.org/book/ch16-00-concurrency.html)
- [Tokio Documentation](https://tokio.rs/)
- [std::thread API](https://doc.rust-lang.org/std/thread/)
- [std::sync API](https://doc.rust-lang.org/std/sync/)

---

**Last Updated:** February 2026  
**Project:** belajar-rust-concurrency
