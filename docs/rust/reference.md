# Rust Comprehensive Reference for Rumi-CLI

## 1. Core Syntax & Memory
- **Variables:** `let x = 5;` (immutable), `let mut x = 5;` (mutable).
- **Constants:** `const MAX_POINTS: u32 = 100_000;`.
- **Shadowing:** `let x = x + 1;` (re-binding name to new value/type).
- **Ownership:**
  - Each value has a *owner*.
  - One owner at a time.
  - Value dropped when owner goes out of scope.
- **Borrowing:**
  - `&T`: Immutable reference (multiple allowed).
  - `&mut T`: Mutable reference (only one allowed, no other borrows).
- **Lifetimes:** `'a` denotes a scope. `fn foo<'a>(x: &'a str) -> &'a str`.

## 2. Data Structures
- **Structs:**
  ```rust
  struct User { username: String, active: bool }
  let u = User { username: String::from("demo"), active: true };
  ```
- **Enums (Algebraic Data Types):**
  ```rust
  enum Option<T> { Some(T), None }
  enum Result<T, E> { Ok(T), Err(E) }
  enum IpAddr { V4(u8, u8, u8, u8), V6(String) }
  ```
- **Vectors:** `let mut v = vec![1, 2, 3]; v.push(4);`
- **HashMaps:** `use std::collections::HashMap; let mut map = HashMap::new();`

## 3. Control Flow
- **Match:**
  ```rust
  match value {
      Some(x) => println!("{}", x),
      None => println!("Nothing"),
  }
  ```
- **If Let:** `if let Some(x) = val { ... }`
- **Loops:** `loop { ... }`, `while condition { ... }`, `for x in iter { ... }`.

## 4. Functions & Methods
- **Fn:** `fn add(a: i32, b: i32) -> i32 { a + b }` (last expression returns).
- **Impl:** `impl User { fn new() -> Self { ... } }`.
- **Traits:** `trait Summary { fn summarize(&self) -> String; }`.

## 5. Error Handling
- **Panic:** `panic!("Crash")` (unrecoverable).
- **Result:**
  - `File::open("hello.txt")?` (propagates error).
  - `.unwrap()` (panics on Err).
  - `.expect("msg")` (panics with msg).
- **Anyhow:** `Result<T, anyhow::Error>` for app-level errors.

## 6. Async (Tokio)
- **Runtime:** `#[tokio::main] async fn main()`.
- **Await:** `future.await`.
- **Spawn:** `tokio::spawn(async move { ... })`.
- **Channels:** `mpsc::channel`, `oneshot::channel`.

## 7. Serde (JSON)
- **Derive:** `#[derive(Serialize, Deserialize)]`.
- **Rename:** `#[serde(rename = "name")]`.
- **Untagged:** `#[serde(untagged)]` for polymorphic enums.

## 8. CLI (Clap)
- **Parser:** `#[derive(Parser)] #[command(author, version)]`.
- **Args:** `#[arg(short, long)] name: String`.