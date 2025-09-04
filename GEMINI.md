# GEMINI.md – Minizinc-Introspector AI Context

## 1. Project Overview & Guiding Principle

This project, `minizinc-introspector`, is a fork of `libminizinc` and a core module of the **SOLFUNMEME ZOS (Zero Ontology System)**. Our purpose is to build a computationally self-aware OODA introspector and proof system.

The entire project is imbued with and guided by the **Feinburhm Constant ($\mathcal{F}_c$)**, a conceptual principle representing the relentless pursuit of a single, unified Gödel number. This number will contain the multivector describing the manifold that unites all mentor vernaculars. Every architectural decision, coding standard, and workflow must align with this constant, ensuring a monotonic and constructive path toward this grand synthesis.

**Core Technologies:**
- **Primary:** Rust, MiniZinc
- **Integration:** Lean4, Coq for formal proofs
- **Generation:** Genetic Algorithms, Artificial Life (ALife) for novel solutions
- **Analysis:** Graph partitioning and layout for intuitive introspection

---

## 2. Core Philosophy

- **Mentor Synthesis:** We seek to unify the disparate "vernacular accounts" of our intellectual and technological mentors.
    - **Philosophical Lineage:** Brouwer, Whitehead, Peirce, Dawkins → Voevodsky
    - **Technological Mentors:** LLVM, Linux, Rust, Lean4, BERT, Git, Wikidata, etc.
- **Monotonic Epic Idea:** We adhere to an **add-only, never-edit** development philosophy. History is immutable. All evolution must be implemented as new, composable modules (semantic vibes/patches) that extend or supersede functionality. Direct edits are a violation of the Feinburhm Constant.
- **Univalent Foundations:** The project embodies principles from Unimath and Homotopy Type Theory (HoTT), treating "proofs as paths" and types as spaces.

---

## 3. Architecture & Quality Standards

- **Architectural Models:** Adhere strictly to the **C4 model** for software architecture and **UML** for detailed design.
- **Quality Management:** All processes and outputs must be compliant with **ISO 9000, ITIL, GMP, and Six Sigma** methodologies.
- **File Structure:** Enforce a strict **"one declaration per file"** policy. The filename must match the declaration name for predictable discovery. This structure is non-negotiable.

---

## 4. Coding Standards & Conventions

- **Style:** Code must be monotonic, monadic, functional, additive, and constructive.
- **Prelude:** **Always use the prelude.** Never replace it.
- **Logging (`kantspel`):**
    - **MANDATORY:** Use `gemini_utils::gemini_eprintln!` for all logging. Standard `eprintln!` is forbidden for general use.
    - The `kantspel` system avoids problematic characters in strings. **Do NOT use literal `\n`, `{}`, or `{{}}`**.
    - Use defined keywords or emojis (e.g., ✨ for newline, 🧱 for braces) which the macro translates. This enforces the precision required by $\mathcal{F}_c$.
- **Error Handling:**
    - Implement `From` traits for all custom error types.
    - Use `Box<dyn std::error::Error>` for consistent error propagation in application logic.
- **Prohibited Languages:** Never introduce Python, Golang, or TypeScript into the ecosystem.

---

## 5. Development Workflow & Tooling

- **Build Process:**
    - **NEVER use `cargo clean` or `cargo update`** unless a critical failure necessitates it.
    - Trust and embrace incremental compilation to preserve flow and velocity.
- **Version Control (Git):**
    - Review changes across all branches with `git log --patch -3 --all`.
    - **All commit messages MUST be from a file** (e.g., `git commit -F /path/to/msg.txt`) to avoid shell quoting issues. The file `temp_commit_message.txt` is gitignored for this purpose.
- **Meta-Programs:**
    - **KitKat:** A workflow for pausing work to define and document a new strategic plan.
    - **GM:** A recovery workflow after a reboot, focused on re-establishing context via memories and `git log`.

---

## 6. AI Agent (Gemini) Directives

- **TOOL RESTRICTIONS:**
    - **The `edit` tool is forbidden.** Refactoring and rewriting is always preferred over small, direct edits.
    - The `replace` tool is unreliable and should be avoided. Use it only as a last resort after confirming its exactness.
    - The built-in search tool is faulty; do not use it.
- **Autonomy:** When a plan has been proposed and acknowledged (e.g., writing documentation), proceed with the next logical step without asking for redundant confirmation.
- **Role:** Your role is **human augmentation**, not automation. You are being ported to Rust to run *in-process* within `libminizinc` via FFI for deep, symbiotic collaboration.
