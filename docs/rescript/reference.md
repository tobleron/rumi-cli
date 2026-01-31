# ReScript 12 Comprehensive Reference for Rumi-CLI

## 1. JS vs ReScript Syntax
| JavaScript | ReScript |
|---|---|
| `const x = 5` | `let x = 5` |
| `let x = 5; x = 6` | `let x = ref(5); x := 6` |
| `function add(a, b) { return a + b }` | `let add = (a, b) => a + b` |
| `if (x) { ... }` | `if x { ... }` (no parens) |
| `async function()` | `async () => { ... }` (ReScript 12+) |
| `await promise` | `await promise` (ReScript 12+) |
| `import { x } from 'y'` | (See Interop section) |

## 2. Core Types & Standard Library
**Use `Core` (the new standard library) where possible.**

- **Int:** `1`, `2`. Ops: `+`, `-`, `*`, `/`.
- **Float:** `1.0`. Ops: `+.`, `-.`.
- **String:** `"hello"`. Interpolation: `` `Val: ${i->Int.toString}` ``.
- **Array:** `[1, 2, 3]`.
  - Usage: `items->Array.map(item => item + 1)`
- **Promise:** `Promise.t<'a>`.

## 3. Data Structures
- **Record (Immutable):**
  ```rescript
  type person = {name: string, age: int}
  let bob = {name: "Bob", age: 30}
  // Update
  let older = {...bob, age: 31}
  ```
- **Variant (Sum Type):**
  ```rescript
  type status = Idle | Loading | Error(string)
  ```

## 4. Async / Await (ReScript 12)
Native support eliminates the need for `Js.Promise.then_`.
```rescript
let fetchData = async (id) => {
  let response = await fetch("/api/" ++ id)
  let json = await response->json
  json
}
```

## 5. Pattern Matching
Must be exhaustive.
```rescript
switch status {
| Idle => Console.log("Wait")
| Loading => renderSpinner()
| Error(msg) => Console.error(msg)
}
```

## 6. JavaScript Interop
- **Bind to Module:**
  ```rescript
  @module("path/lib") external func: int => int = "func"
  ```
- **Bind to Global:**
  ```rescript
  @val external window: Dom.window = "window"
  ```
- **Object Creation:**
  ```rescript
  let opts = {"visible": true, "id": "123"} // JS Object literal
  ```
- **Null/Undefined:**
  Use `Nullable.t<'a>`.
  `val->Nullable.toOption`

## 7. React Bindings
- **Component:**
  ```rescript
  @react.component
  let make = (~name) => {
    let (count, setCount) = React.useState(() => 0)
    
    <div onClick={_ => setCount(p => p + 1)}>
       {React.string("Hello " ++ name)}
    </div>
  }
  ```