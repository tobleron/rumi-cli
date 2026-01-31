# ReScript Quick Reference for Rumi-CLI

## Core Syntax
- **Variables:** `let x = 10` (immutable), `let x = ref(10)` (mutable via `x.contents = 20`).
- **Functions:** `let add = (a, b) => a + b`.
- **Types:** `type person = {name: string, age: int}`.
- **Variants:** `type result = Success(string) | Error(string)`.

## Pattern Matching
```rescript
match result {
| Success(msg) => Js.log("Yay: " ++ msg)
| Error(err) => Js.log("Darn: " ++ err)
}
```

## JS Interop
- **External:** `@val external alert: string => unit = "alert"`.
- **JSON:** Use `@spice` or `Js.Json.parseExn`.

## Common Pitfalls
- Use `++` for string concatenation (not `+`).
- Use `==` for structural equality (most common).
- Labels: `let f = (~name) => name`. Call with `f(~name="Bob")`.
