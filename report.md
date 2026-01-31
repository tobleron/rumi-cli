# **Structural Search and Transformation Architectures for ReScript: A Deep Research Report on ast-grep Implementation and Alternatives**

## **1\. Executive Summary**

The transition from lexical, regular-expression-based refactoring to structural, Abstract Syntax Tree (AST)-based transformation represents a critical maturation in software maintenance, particularly for strongly typed functional languages like ReScript. This report provides an exhaustive analysis of the implementation challenges associated with adopting ast-grep (sg) for ReScript projects, diagnosing the specific dissonance between the tool’s default metavariable syntax and the strict lexical grammar enforced by the tree-sitter-rescript parser.

The investigation confirms that the user's inability to match patterns such as Js.log($MSG) stems from a fundamental parsing constraint: the dollar sign ($), while standard for metavariables in many languages, is rejected by the ReScript parser as an invalid identifier start character, resulting in "ERROR nodes" within the search pattern's Concrete Syntax Tree (CST). Because ast-grep relies on an isomorphic match between the pattern CST and the source CST, the presence of these error nodes causes the search engine to abort matches against valid, error-free source code.

This report establishes three primary remediation pathways. First, and most effectively, the **Metavariable Syntax Alignment** strategy involves reconfiguring ast-grep to utilize the underscore (\_) as the "expando character." Unlike $, the underscore is a valid start character for ReScript identifiers, allowing patterns like Js.log(\_MSG) to parse as valid code while still functioning as capture groups. Second, the **Contextual Pattern Injection** strategy utilizes ast-grep's context configuration to bypass top-level parsing restrictions, useful for fragment matching. Third, the report evaluates **Comby** as a high-viability alternative. Comby's "language-aware fuzzy matching" architecture avoids the strict parsing requirements of ast-grep, offering a robust fallback for scenarios where CST fidelity is secondary to simple structural replacement.

The document concludes with a comprehensive implementation guide, synthesizing community findings from the rescript-lint project into a production-ready configuration. This includes the exact sgconfig.yml settings, corrected rule definitions, and a comparative analysis of alternative tools to ensure the "intelligent" refactoring objective is met with precision.

## ---

**2\. Foundations of Structural Search in Strictly Typed Languages**

### **2.1 The Evolution from Regex to CST Matching**

The software engineering discipline has long relied on Regular Expressions (Regex) for text processing. While effective for simple string manipulation, Regex operates on a linear stream of characters, oblivious to the hierarchical and recursive nature of programming languages. In a language like ReScript, which features nested blocks, curried functions, and complex type definitions, Regex struggles to distinguish between semantically identical structures that differ in formatting (e.g., whitespace, line breaks) or to safely scope changes to specific code blocks.2

Structural Search and Replace (SSR) tools, such as ast-grep, represent a paradigm shift. Instead of treating code as text, they treat it as a tree—specifically, a Concrete Syntax Tree (CST) or Abstract Syntax Tree (AST). ast-grep utilizes the Tree-Sitter parsing library to generate these trees.3 When a user searches for Js.log($A), the tool does not look for the characters J, s, ., l, o, g. Instead, it looks for a call\_expression node where the function is a member expression Js.log and the arguments list contains a node matching the metavariable $A.4 This allows the tool to ignore formatting differences and respect syntactic boundaries, preventing common Regex errors such as matching text inside string literals or comments.

### **2.2 The ReScript Architectural Context**

ReScript occupies a unique niche in the programming landscape. It is a robustly typed language that compiles to efficient, human-readable JavaScript.5 Architecturally, it shares a lineage with OCaml and ReasonML, inheriting a powerful type system and a strict grammar.6 Unlike JavaScript, which has a relatively permissive parser (often accepting missing semicolons or loosely typed identifiers), ReScript enforces a rigid syntax to ensure type safety and compilation correctness.7

This strictness is a double-edged sword for SSR tools. While it ensures that the code being refactored is well-structured, it imposes severe constraints on the "pattern code" used for searching. ast-grep operates on the principle that the search pattern itself must be valid code that can be parsed by the language's grammar.2 If the pattern snippet violates the language's grammar—even slightly—the parser may produce ERROR nodes. In permissive languages like Python or JavaScript, the parser might recover or accept non-standard tokens. In ReScript, strict lexical rules regarding identifier casing (e.g., Module identifiers *must* start with uppercase; variable identifiers *must* start with lowercase or underscore) mean that standard SSR conventions (like uppercase metavariables $VAR) often result in immediate parsing failures.8

### **2.3 The Toolchain Dependency Graph**

The user's environment highlights the dependency chain required for ast-grep to function:

1. **The Core Engine (ast-grep):** A Rust-based binary that handles the traversal logic, pattern compilation, and replacement execution.9  
2. **The Parser Generator (Tree-Sitter):** A library that provides the incremental parsing capability. It generates a parser from a grammar.js specification file.10  
3. **The Language Grammar (tree-sitter-rescript):** The specific set of rules defining valid ReScript syntax. This is compiled into a dynamic shared library (rescript.so).3  
4. **The Configuration (sgconfig.yml):** The bridge file that tells ast-grep how to load the dynamic library and what file extensions to associate with it.3

The failure observed ("Pattern contains an ERROR node") occurs at the interface between components 2 and 3\. The ast-grep engine passes the user's pattern string to the rescript.so parser. The parser rejects the string as invalid ReScript, returning a tree containing ERROR nodes. The core engine then reports this as a warning and fails to match valid code.8

## ---

**3\. Pathology of the Failure: Diagnosing ReScript Parser Conflicts**

To understand why the user's patterns failed, we must perform a forensic analysis of the tree-sitter-rescript grammar rules and their interaction with ast-grep's default behavior.

### **3.1 Failure Case 1: The $VAR Identifier Conflict**

The user attempted to match Js.log($MSG). In ast-grep, the $ symbol is the default "expando character"—a prefix that marks a token as a metavariable (a wildcard that captures content).2

**Lexical Analysis:** The ReScript grammar defines strict rules for identifiers.11

* **Value Identifiers (identifier):** Must start with a lowercase letter (a-z) or an underscore (\_). Example: myVariable, \_unused.  
* **Module Identifiers (module\_identifier):** Must start with an uppercase letter (A-Z). Example: Console, React.  
* **Variant Constructors (variant\_identifier):** Must start with an uppercase letter, often used in pattern matching. Example: None, Some, \#Red.

**The Parsing Event:**

When tree-sitter-rescript encounters $MSG:

1. The scanner reads $.  
2. It checks if $ is a valid start character for any known token type (Identifier, Keyword, Operator).  
3. In ReScript, $ is not a valid start for a standard identifier. It is not an operator in this context (unlike \+ or \-).  
4. The parser cannot assign a valid token type (like identifier) to $MSG.  
5. **Result:** The parser emits an ERROR node wrapping the $ token, flagging it as an unexpected character.

**The Structural Mismatch:**

ast-grep attempts to match this pattern tree against the source tree of Js.log("Hello").

* **Pattern Tree:** call\_expression \-\> function: Js.log \-\> arguments: (ERROR)  
* **Source Tree:** call\_expression \-\> function: Js.log \-\> arguments: (string\_literal)

Since ERROR does not match string\_literal, the search fails. The user's observation ("Pattern contains an ERROR node") is the direct consequence of this strict lexical rejection.8

### **3.2 Failure Case 2: The @VAR Attribute Conflict**

The user hypothesized a conflict and tried changing expandoChar to @. This resulted in the parser seeing a variant\_identifier or ERROR.

**Lexical Analysis:** In ReScript, the @ character is reserved for **Attributes** (decorators).13

* **Syntax:** @attributeName or @module("path").  
* **Placement:** Attributes attach to the *preceding* or *following* node (e.g., a function definition or a type).

**The Parsing Event:**

When parsing Js.log(@A):

1. The parser recognizes @ as the start of an attribute.  
2. It parses @A as an attribute node (or potentially a decorator).  
3. However, attributes are typically not valid as direct arguments to a function call in this syntax. The parser might force it into an attribute node structure.

**The Structural Mismatch:**

* **Pattern Tree:** call\_expression \-\> arguments: (attribute)  
* **Source Tree:** call\_expression \-\> arguments: (string\_literal)

Even if the pattern parses without an ERROR node (which is unlikely given the context restrictions of attributes), the **Node Kind** will be attribute. An attribute node is structurally distinct from an expression node. ast-grep matches based on node kind; therefore, an attribute metavariable will never match a string argument. This explains the user's result: "Matches nothing" or "variant\_identifier".8

### **3.3 Failure Case 3: The let make \= () \=\> { $$$ } Block Conflict**

The user attempted to match a function body using $$$ (the "match remaining" or "any sequence" operator).

**Grammar Constraints:** In ReScript, the body of a function {... } is a **Block**. A block contains a sequence of **Statements** or **Expressions**.14

* The token $$$ (or even ...) is not a valid statement in ReScript.  
* When the parser encounters { $$$ }, it expects a valid expression (e.g., Js.log("hi"), let x \= 1).  
* Finding $$$, it throws a syntax error, generating an ERROR node for the block content.

**The Structural Mismatch:**

The pattern expects a block containing an error. The source code contains a block containing valid statements. They do not match.

## ---

**4\. Primary Solution Strategy: Metavariable Syntax Alignment**

The central finding of this research is that successful structural search in ReScript depends on aligning the ast-grep configuration with the ReScript grammar's definition of a valid identifier.

### **4.1 The Underscore (\_) Heuristic**

The most effective solution is to configure ast-grep to use the underscore (\_) as the expandoChar.

**Why the Underscore Works:**

1. **Lexical Validity:** In ReScript, an identifier *can* start with an underscore. The syntax \_MSG is treated by the parser as a valid identifier (often used for ignored variables or private bindings).15  
2. **Parser Acceptance:** When the pattern Js.log(\_MSG) is submitted, tree-sitter-rescript parses \_MSG as a standard identifier node. It does not produce an ERROR.  
3. **Metavariable Mechanics:** ast-grep scans the CST. It sees an identifier node with the text \_MSG. Because expandoChar is set to \_, ast-grep recognizes this as a metavariable named MSG. It then "captures" the corresponding node in the source tree (e.g., the string\_literal "Hello").3

### **4.2 Implementing the Underscore Configuration**

To implement this, the sgconfig.yml file must be updated to explicitly define the expandoChar for the ReScript language definition.

#### **4.2.1 The Configuration File (sgconfig.yml)**

YAML

\# sgconfig.yml  
ruleDirs:  
  \- rules

customLanguages:  
  rescript:  
    \# Path to the compiled dynamic library.   
    \# Ensure this matches the user's compilation artifact location.  
    libraryPath:./rescript.so  
      
    \# File extensions to associate with this language  
    extensions: \[res, resi\]  
      
    \# CRITICAL SETTING:   
    \# Use underscore to allow metavariables to parse as identifiers.  
    \# This replaces the default '$'.  
    expandoChar: \_ 

**Note on Library Path:** The path ./rescript.so is relative to the sgconfig.yml file. Users must ensure the binary compiled via gcc is moved to the project root or referenced via an absolute path.3

#### **4.2.2 Rule Redefinition**

With expandoChar: \_, all rules must be updated to use \_ instead of $. The ellipsis operator ($$$) becomes three underscores (\_\_\_).

**Original User Rule (Fixed):**

* **Old:** Js.log($MSG)  
* **New:** Js.log(\_MSG)

**Multi-Line Match Rule:**

* **Old:** let make \= () \=\> { $$$ }  
* **New:** let make \= () \=\> { \_\_\_ }

### **4.3 Validation of the Fix**

Let us trace the execution of the fixed pattern Js.log(\_MSG) against the source Js.log("Error 404").

1. **Pattern Parsing:** ast-grep sends Js.log(\_MSG) to the parser.  
   * Js \-\> module\_identifier  
   * log \-\> property\_identifier  
   * \_MSG \-\> identifier (Valid\!)  
   * **Result:** A clean CST with call\_expression at the root.  
2. **Source Parsing:** ast-grep parses source Js.log("Error 404").  
   * "Error 404" \-\> string\_literal.  
3. **Matching:**  
   * Root call\_expression matches.  
   * Function Js.log matches.  
   * Argument position 1: Pattern has identifier (metavariable MSG). Source has string\_literal.  
   * **Capture:** ast-grep captures "Error 404" into metavariable MSG.  
4. **Transformation:**  
   * Fix pattern: Console.log(\_MSG).  
   * Output: Console.log("Error 404").

This strategy resolves the parse error completely by strictly adhering to the grammar's requirements.8

## ---

**5\. Secondary Solutions: Contextual Injection and Constraints**

While the underscore strategy is robust, specific edge cases (e.g., matching code fragments that are not valid top-level statements) may require advanced features like Contextual Pattern Objects.

### **5.1 Contextual Pattern Injection**

The ReScript parser expects complete definitions at the top level. A snippet like switch data {... } might fail if not wrapped in a function or block, depending on strictness. ast-grep allows defining a context to wrap the pattern, tricking the parser into accepting a fragment.16

**Problem:** Matching a specific argument inside a function call without matching the whole call.

**Pattern:** \_ARG (might fail if \_ARG isn't valid at top level, though usually it is).

**Solution:** Use the pattern object syntax in the YAML rule.

YAML

id: argument-replacement  
language: rescript  
rule:  
  pattern:  
    \# Context wraps the metavariable in a valid structure  
    context: 'Js.log(\_ARG)'  
    \# Selector extracts only the node corresponding to \_ARG for matching  
    selector: arguments  
fix: 'NewWrapper(\_ARG)'

This technique forces the parser to process Js.log(\_ARG) (which we know is valid) and then focuses the matching engine solely on the arguments node. This is highly effective for "sub-tree" matching where the isolated snippet might be ambiguous (e.g., distinguishing between a block {} and a record {}).17

### **5.2 Kind-Based Constraints (The "No-Variable" Approach)**

If a user cannot find *any* character that parses as a valid metavariable (unlikely in ReScript but possible in other strict DSLs), the fallback is to use generic identifier matching combined with constraints.

**Strategy:** Use a literal identifier like PLACEHOLDER and rely on ast-grep's inability to distinguish identifiers without constraints—except ast-grep by default matches exact text for identifiers. To bypass this, we use the kind constraint.

YAML

id: hard-mode-match  
language: rescript  
rule:  
  kind: call\_expression  
  field: function  
    pattern: Js.log  
  field: arguments  
    \# We match ANY node that is an argument, regardless of text  
    kind: arguments 

**Limitation:** This method can *find* nodes but cannot *capture* the content of the argument for replacement (the fix string needs a captured metavariable). Therefore, this is useful for Linting (finding bad code) but not for Refactoring (rewriting code) unless the replacement is static. For the user's objective (Refactoring), this strategy is insufficient compared to the Underscore solution.8

## ---

**6\. Alternative Ecosystems: A Comparative Audit**

If ast-grep proves recalcitrant despite configuration tuning, it is necessary to evaluate alternative tools. The user specifically inquired about comby and gritql.

### **6.1 Comby: The Language-Aware Fuzzy Matcher**

**Architecture:**

Comby operates on a different principle than ast-grep. It does not parse code into a rigorous CST. Instead, it tokenizes the stream, recognizing balanced delimiters (parentheses (), braces {}, brackets \`\`) and string literals. It is "language-aware" in that it respects these boundaries but "fuzzy" in that it ignores invalid syntax between them.

**Suitability for ReScript:**

ReScript's syntax (C-like) is highly compatible with Comby's generic matchers (.res is not a default, but .cs or .c often work). Because Comby does not enforce a grammar, it **never** suffers from the "ERROR node" problem. Js.log(:) is simply a pattern of "text Js.log followed by parentheses containing *something*."

**Configuration for User's Problem:**

To replace Js.log with Console.log:

Bash

comby 'Js.log(:\[args\])' 'Console.log(:\[args\])'.res

**Pros:**

* **Zero Configuration:** No grammar.js, no compilation of .so files.  
* **Robustness:** Ignores syntax errors in the rest of the file.  
* **Ease of Use:** Metavariable syntax :\[var\] is extremely unlikely to conflict.

**Cons:**

* **Lack of Semantic Precision:** Cannot easily ask "only replace Js.log if it is inside a module named User" without complex rule chaining.  
* **No LSP Integration:** Unlike ast-grep, it doesn't plug into VSCode for real-time linting as easily.

**Verdict:** Comby is the **Strongest Alternative** for one-off refactoring tasks where the strict setup of ast-grep is a bottleneck.

### **6.2 GritQL: The State-Aware Matcher**

**Architecture:** GritQL is a newer query language designed for heavy state management and complex rewriting logic (e.g., "if I change this function signature, update all callers and imports").18

**Suitability for ReScript:**

GritQL currently has limited first-class support for ReScript compared to major languages like TypeScript or Python. While powerful, utilizing it for ReScript would likely require defining a custom grammar mapping similar to ast-grep, incurring the same setup costs without the maturity of the tree-sitter-rescript integration that ast-grep already facilitates.

### **6.3 Semgrep: The Semantic Matcher**

**Architecture:** Semgrep (Semantic Grep) also uses Tree-Sitter but abstracts patterns further. It is written in OCaml 19, which is ironically the parent language of ReScript.

**Suitability for ReScript:** Semgrep has experimental support for many languages but does not list ReScript as a "GA" (General Availability) language with a polished ruleset registry.20 Adding support requires writing OCaml code to map the CST to Semgrep's internal generic AST. This is a significantly higher barrier to entry than ast-grep's configuration-only approach.

## ---

**7\. The Community Standard: rescript-lint Analysis**

Research into the ReScript ecosystem reveals an existing project, rescript-lint, authored by user jderochervlk and others.21 This project functions as a proof-of-concept for ast-grep in ReScript.

**Architectural Insights from rescript-lint:**

1. **Local Binary Compilation:** The project includes scripts to compile tree-sitter-rescript locally, acknowledging the difficulty of distributing pre-compiled binaries for custom languages.22  
2. **Config Encapsulation:** It wraps the sgconfig.yml inside the node\_modules package, allowing users to run sg scan pointing to the library's config.  
3. **Rule Definitions:** The community rules explicitly utilize the underscore expando pattern, confirming the hypothesis derived in Section 4\.

**Consensus:** The community has converged on using ast-grep with a custom-compiled grammar and underscore metavariables as the standard for linting and refactoring.

## ---

**8\. Implementation Guide: The "Universal" Configuration**

This section provides the complete, copy-paste artifacts required to resolve the user's problem, integrating all findings into a cohesive solution.

### **8.1 Step 1: Parser Compilation**

The user must ensure the shared library is compiled with the scanner, as ReScript relies on external scanning for advanced syntax (like nested comments or template literals).

**Command (Linux/GCC):**

Bash

\# Assuming user is in the tree-sitter-rescript directory  
\# \-I src: Include headers  
\# src/parser.c src/scanner.c: Compile BOTH parser and scanner  
gcc \-o rescript.so \-shared src/parser.c src/scanner.c \-I src \-fPIC

### **8.2 Step 2: The sgconfig.yml**

Create this file in the project root.

YAML

\# sgconfig.yml  
ruleDirs:  
  \- rules

customLanguages:  
  rescript:  
    \# Adjust path if the.so is in a subfolder  
    libraryPath:./rescript.so  
    extensions: \[res, resi\]  
    \# THE FIX: Underscore expando  
    expandoChar: \_ 

### **8.3 Step 3: The Refactoring Rule**

Create rules/refactor\_log.yml.

YAML

id: refactor-js-log-to-console  
language: rescript  
severity: warning  
message: "Deprecating Js.log in favor of Console.log"  
rule:  
  \# Pattern matches Js.log(any\_argument)  
  pattern: Js.log(\_MSG)  
fix: Console.log(\_MSG)

### **8.4 Step 4: The Complex Pattern Fix**

For the user's failed pattern let make \= () \=\> { $$$ }:

YAML

id: match-component-body  
language: rescript  
rule:  
  \# Context implies this is a top-level binding  
  pattern: |  
    let make \= () \=\> {  
      \_\_\_BODY  
    }

*Note: We use \_\_\_ (three underscores) to match the sequence of statements inside the block.*

## ---

**9\. Conclusion**

The "ERROR node" failures encountered when implementing ast-grep for ReScript are not defects in the tool or the parser, but rather symptoms of a configuration mismatch. ReScript's strict grammar rejects the standard $ metavariable syntax, breaking the isomorphic matching engine.

The solution is definitive: **adopt the underscore (\_) as the expando character.** This aligns the tooling with the language's lexical rules, allowing patterns to parse as valid identifiers. While alternative tools like **Comby** offer a lower-friction path for simple text replacement via fuzzy parsing, ast-grep—properly configured—remains the superior choice for semantic, type-aware, and structure-sensitive refactoring in the ReScript ecosystem. The existence of rescript-lint validates this approach, providing a template for scalable adoption.

### **Summary of Recommendations**

| Requirement | Recommendation | Reason |
| :---- | :---- | :---- |
| **Metavariable Syntax** | Use \_VAR (Underscore) | Valid ReScript identifier; avoids Parse Errors. |
| **Config Setting** | expandoChar: \_ | Enables \_ as the capture prefix. |
| **Tooling Choice** | ast-grep | For robust, structure-aware refactoring. |
| **Fallback Tool** | Comby | For rapid, fuzzy text replacement if strict parsing fails. |
| **Parser Build** | Include scanner.c | Essential for full grammar coverage. |

**Citations:** 1

#### **Works cited**

1. Comby · Structural code search and replace for \~every language., accessed January 31, 2026, [https://comby.dev/](https://comby.dev/)  
2. Pattern Syntax | ast-grep, accessed January 31, 2026, [https://ast-grep.github.io/guide/pattern-syntax.html](https://ast-grep.github.io/guide/pattern-syntax.html)  
3. Custom Language Support | ast-grep, accessed January 31, 2026, [https://ast-grep.github.io/advanced/custom-language.html](https://ast-grep.github.io/advanced/custom-language.html)  
4. Core Concepts in ast-grep's Pattern, accessed January 31, 2026, [https://ast-grep.github.io/advanced/core-concepts.html](https://ast-grep.github.io/advanced/core-concepts.html)  
5. ReScript is a robustly typed language that compiles to efficient and human-readable JavaScript. \- GitHub, accessed January 31, 2026, [https://github.com/rescript-lang/rescript](https://github.com/rescript-lang/rescript)  
6. What is actually going on now with ReasonML and ReScript? \- Ecosystem \- discuss OCaml, accessed January 31, 2026, [https://discuss.ocaml.org/t/what-is-actually-going-on-now-with-reasonml-and-rescript/13973](https://discuss.ocaml.org/t/what-is-actually-going-on-now-with-reasonml-and-rescript/13973)  
7. Announcing ReScript 12, accessed January 31, 2026, [https://rescript-lang.org/blog/release-12-0-0/](https://rescript-lang.org/blog/release-12-0-0/)  
8. Frequently Asked Questions \- ast-grep, accessed January 31, 2026, [https://ast-grep.github.io/advanced/faq.html](https://ast-grep.github.io/advanced/faq.html)  
9. ast-grep | structural search/rewrite tool for many languages, accessed January 31, 2026, [https://ast-grep.github.io/](https://ast-grep.github.io/)  
10. Tree-sitter: Introduction, accessed January 31, 2026, [https://tree-sitter.github.io/](https://tree-sitter.github.io/)  
11. Overview | ReScript Language Manual, accessed January 31, 2026, [https://rescript-lang.org/docs/manual/overview/](https://rescript-lang.org/docs/manual/overview/)  
12. Function | ReScript Language Manual, accessed January 31, 2026, [https://rescript-idea.github.io/docs/manual/latest/function](https://rescript-idea.github.io/docs/manual/latest/function)  
13. Attribute (Decorator) | ReScript Language Manual, accessed January 31, 2026, [https://rescript-lang.org/docs/manual/attribute/](https://rescript-lang.org/docs/manual/attribute/)  
14. ReScript: Rust like features for JavaScript \- DEV Community, accessed January 31, 2026, [https://dev.to/jderochervlk/rescript-rust-like-features-for-javascript-27ig](https://dev.to/jderochervlk/rescript-rust-like-features-for-javascript-27ig)  
15. Pattern Matching / Destructuring | ReScript Language Manual, accessed January 31, 2026, [https://rescript-lang.org/docs/manual/pattern-matching-destructuring/](https://rescript-lang.org/docs/manual/pattern-matching-destructuring/)  
16. Atomic Rule \- ast-grep, accessed January 31, 2026, [https://ast-grep.github.io/guide/rule-config/atomic-rule.html](https://ast-grep.github.io/guide/rule-config/atomic-rule.html)  
17. Deep Dive into ast-grep's Pattern Syntax, accessed January 31, 2026, [https://ast-grep.github.io/advanced/pattern-parse.html](https://ast-grep.github.io/advanced/pattern-parse.html)  
18. Typed languages are better suited for vibecoding \- Hacker News, accessed January 31, 2026, [https://news.ycombinator.com/item?id=44780878](https://news.ycombinator.com/item?id=44780878)  
19. Add support for a new language \- Semgrep, accessed January 31, 2026, [https://semgrep.dev/docs/contributing/adding-a-language](https://semgrep.dev/docs/contributing/adding-a-language)  
20. List of Languages with Built-in Support | ast-grep, accessed January 31, 2026, [https://ast-grep.github.io/reference/languages.html](https://ast-grep.github.io/reference/languages.html)  
21. jderochervlk \- NPM, accessed January 31, 2026, [https://www.npmjs.com/\~jderochervlk](https://www.npmjs.com/~jderochervlk)  
22. I'm looking into using ast-grep fo linting ReScript. What rules should I investigate?, accessed January 31, 2026, [https://forum.rescript-lang.org/t/im-looking-into-using-ast-grep-fo-linting-rescript-what-rules-should-i-investigate/5975](https://forum.rescript-lang.org/t/im-looking-into-using-ast-grep-fo-linting-rescript-what-rules-should-i-investigate/5975)