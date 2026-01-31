# Problem Report: Implementing Structural Search and Replace for ReScript using ast-grep

## Objective
We are attempting to use `ast-grep` (sg) for structural search and transformation within a ReScript 12 project. The goal is to perform "intelligent" refactoring (e.g., replacing `Js.log($A)` with `Console.log($A)`) that understands the code's Abstract Syntax Tree (AST) rather than relying on brittle regex or literal string matching.

## Environment Setup
1. **Tool:** `ast-grep` installed via `cargo`.
2. **Parser:** Cloned the official `rescript-lang/tree-sitter-rescript` grammar and compiled it into a shared library (`rescript.so`) using `gcc`.
3. **Configuration (`sgconfig.yml`):**
   ```yaml
   customLanguages:
     rescript:
       libraryPath: /path/to/parsers/rescript.so
       extensions: [res, resi]
   ```

## Attempts and Failures

### 1. Simple CLI Patterns
We tried matching common ReScript structures using the standard `ast-grep` metavariable syntax (`$VAR` and `$$$`):
*   **Pattern:** `let make = () => { $$$ }`
    *   **Result:** Warning: "Pattern contains an ERROR node". Matches nothing.
*   **Pattern:** `Js.log($MSG)` or `Js.log($$$ARGS)`
    *   **Result:** No match found, even when the code contains exactly `Js.log("string")`.
*   **Pattern:** `$A`
    *   **Result:** "No AST root is detected."

### 2. Literal Matching
*   **Pattern:** `Js.log("Hello from Rumi!")`
    *   **Result:** **Success**. This confirms the parser is correctly loading and reading the file. However, literal matching defeats the purpose of structural search.

### 3. YAML Rule Definitions
We attempted to use `kind` and `field` constraints in YAML rules to bypass the pattern parser errors:
*   **Approach:** Filtered by `kind: call_expression` and checked if the `function` field matches `Js.log`.
    *   **Result:** We successfully *identified* the calls using `kind: call_expression`, but could not capture the arguments into a metavariable for use in a `fix` (transformation).
*   **Error:** When using `pattern: Js.log($$$ARGS)` inside a rule, it either fails to match or reports "Undefined meta var `ARGS` used in `fix`."

### 4. Expando Character Conflicts
Hypothesizing a conflict between `ast-grep`'s `$` and ReScript's syntax, we tried changing the `expandoChar` in `sgconfig.yml`:
*   **Tried:** `expandoChar: "@"` or `expandoChar: "$"`
*   **Result:** Using `@A` resulted in the parser seeing a `variant_identifier` or `ERROR` node instead of a metavariable.

## Key Observations
*   The ReScript Tree-sitter grammar appears to be very strict about what constitutes a "valid" snippet for a pattern. Most snippets result in `ERROR` nodes when parsed as standalone patterns.
*   The `debug-query` output shows that patterns like `let $A = $B` are being parsed as `ERROR` nodes containing the `let` and `=` tokens, rather than correctly identifying the `let_binding` structure.
*   `ast-grep` successfully parses the source files, but the "bridge" between the pattern-matching logic and the ReScript AST nodes is failing for dynamic metavariables.

## Request for Research
Please research the consensus and most appropriate way to achieve structural search and replace for ReScript. Specifically:
1. Is there a specific `ast-grep` configuration or version of the ReScript parser known to work?
2. Are there alternative structural tools (e.g., `comby`, `gritql`) that have better first-class support for ReScript's specific AST structure?
3. How can we correctly escape or define metavariables in `ast-grep` so they are not swallowed by the ReScript parser as syntax errors or variants?
4. Is there an existing `sgconfig.yml` or rule set used by the ReScript community for `ast-grep`?
