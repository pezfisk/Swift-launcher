# Understanding WASM Architecture in Swift Launcher

## Table of Contents
1. [What is WASM?](#what-is-wasm)
2. [Why WASM for Plugins?](#why-wasm-for-plugins)
3. [The Component Model](#the-component-model)
4. [Swift Launcher's WASM Architecture](#swift-launchers-wasm-architecture)
5. [The Plugin Flow Diagram](#the-plugin-flow-diagram)
6. [How Information Flows](#how-information-flows)
7. [Memory and Isolation](#memory-and-isolation)
8. [WASI and System Access](#wasi-and-system-access)

---

## What is WASM?

**WebAssembly (WASM)** is a binary instruction format designed to be:

- **Portable** - Run on any CPU (x86, ARM, RISC-V)
- **Fast** - Near-native performance (usually 80-90% of native speed)
- **Sandboxed** - Runs in a restricted environment with no direct OS access
- **Compact** - Binary format is 10x smaller than JavaScript
- **Language-agnostic** - Write plugins in Rust, Python, C++, Go, JavaScript, Zig, and more

### The WASM Stack

```
Your Code (Rust, Python, C, etc.)
        ↓
    Compiler
        ↓
    WASM Binary (.wasm file)
        ↓
    WASM Runtime (Wasmtime)
        ↓
    Native Machine Code
        ↓
    CPU
```

The WASM binary is **portable** - the same `.wasm` file runs on Linux, macOS, Windows, etc. The runtime compiles it to native code for your specific CPU.

---

## Why WASM for Plugins?

### Problems with Traditional Plugins

**Shared Libraries (.so, .dll)**
- Security risk: Can access your entire filesystem and memory
- Stability risk: One crash brings down the host
- Maintenance nightmare: Different binary for each OS
- Dependency hell: Plugin requires specific versions of 20 libraries

**Interpreted Languages (Python scripts)**
- Performance hit: 10-50x slower than native
- Large runtime overhead
- Inconsistent behavior across versions

### WASM's Advantages

1. **Security** - Plugin can't access filesystem/network unless explicitly granted
2. **Stability** - Crash in plugin doesn't crash Swift Launcher
3. **Portability** - Same binary runs on Linux/macOS/Windows/ARM
4. **Performance** - Near-native speed (1-5% overhead)
5. **Polyglot** - Write plugins in any language that compiles to WASM
6. **Versioning** - Each plugin is self-contained, no dependency conflicts

### Real-World Example

**Traditional Plugin Problem:**
```
User has: Python 3.11
Plugin compiled for: Python 3.10
Result: TypeError in plugin (version mismatch)
```

**WASM Plugin Solution:**
```
Plugin compiled once as: echo-plugin.wasm
Works on: Python 3.9, 3.10, 3.11, 3.12, and beyond
No version mismatch possible
```

---

## The Component Model

### What is a Component?

WASM Components are a **higher-level** abstraction built on top of core WASM:

- **Core WASM** = Low-level instruction format (like assembly)
- **Component Model** = Interface layer for multi-language interop

### Interface Definition Language (WIT)

Components use **WIT** (WebAssembly Interface Types) to define contracts:

```wit
interface runner {
    resource action-item {
        get-name: func() -> string
        get-exec: func() -> string
    }
    
    get-trigger: func() -> string
    handle: func(input: string) -> list<action-item>
}
```

This is saying:
- "Components implementing this must have a `get-trigger` function that returns a string"
- "They must have a `handle` function that takes a string and returns a list of action-items"

### Why Components Matter

**Before (Core WASM):** You directly call raw memory addresses
```rust
unsafe { (*0x12345678 as fn(i32) -> i32)(42) }  // Scary!
```

**After (Component Model):** You call typed functions
```rust
let results = runner.handle("input")?;  // Safe, clear, type-checked
```

---

## Swift Launcher's WASM Architecture

### High-Level View

```
┌─────────────────────────────────────────────────────────┐
│                 Swift Launcher                          │
│              (Rust native binary)                       │
└─────────────────────────────────────────────────────────┘
                          ↓
        ┌─────────────────┬─────────────────┐
        ↓                 ↓                 ↓
    ┌────────┐      ┌─────────┐      ┌────────┐
    │ Plugins│      │ Plugins │      │Plugins │
    │Manager │      │Registry │      │Runtime │
    └────────┘      └─────────┘      └────────┘
        ↓                 ↓                ↓
    ┌─────────────────────────────────────────────┐
    │        Wasmtime WASM Runtime                │
    │    (Safe execution environment)             │
    └─────────────────────────────────────────────┘
        ↓         ↓         ↓         ↓
    ┌──────┐  ┌──────┐  ┌──────┐  ┌──────┐
    │echo. │  │calc. │  │dir.  │  │...   │
    │wasm  │  │wasm  │  │wasm  │  │wasm  │
    └──────┘  └──────┘  └──────┘  └──────┘
```

### The Three Layers

#### Layer 1: Plugin Discovery
```rust
// plugins/mod.rs
pub fn discover_plugins() -> Vec<PathBuf> {
    let plugins_dir = "~/.config/swift/plugins";
    std::fs::read_dir(plugins_dir)
        .flat_map(|e| e.ok())
        .filter(|e| e.path().extension() == Some("wasm"))
        .map(|e| e.path())
        .collect()
}
```

Swift Launcher scans `~/.config/swift/plugins/` for all `.wasm` files.

#### Layer 2: Component Initialization
```rust
// plugins/component.rs
let engine = Engine::new(config)?;
let mut linker = Linker::new(&engine);

// Register WASI functions (gives plugins controlled access)
wasmtime_wasi::p2::add_to_linker_sync(&mut linker)?;

// Load the WASM binary
let mut store = Store::new(&engine, wasi_context);
let component = Component::from_file(&engine, "echo.wasm")?;

// Bind the component to our linker
let instance = linker.instantiate_sync(&mut store, &component)?;
```

Each plugin gets its own **sandboxed instance** in the runtime.

#### Layer 3: Function Invocation
```rust
// plugins/host.rs
let guest = runner::Guest::new(&mut store, &instance)?;
let trigger = guest.get_trigger(&mut store)?;
let results = guest.handle(&mut store, input)?;
```

Call the plugin's functions through the Component Model interface.

### The Linker Pattern

The **Linker** is the bridge between host and plugin:

```
Host (Swift Launcher)         Plugin (echo.wasm)
         ↓                             ↓
[I need to run get_trigger]   [I can provide get_trigger]
         ↓                             ↓
    └─────── Linker ──────────────┘
        (matches & connects)
```

The Linker ensures:
- Plugin implements all required functions
- Function signatures match the interface
- Plugin gets only the capabilities it needs (WASI functions)
- Memory is isolated and managed

---

## The Plugin Flow Diagram

### User Types `>hello world`

```
                            ┌─────────────────────┐
                            │ User presses Enter  │
                            └──────────┬──────────┘
                                       ↓
                            ┌─────────────────────────────────┐
                            │ Swift receives input string     │
                            │ ">hello world"                  │
                            └──────────┬──────────────────────┘
                                       ↓
                    ┌──────────────────────────────────────────┐
                    │ Plugin Discovery                         │
                    │ Load all .wasm files from plugins/       │
                    └──────────┬───────────────────────────────┘
                               ↓
                    ┌─────────────────────────────────────────┐
        ┌───────────┤ For each plugin, call get_trigger()    │
        │           └──────────┬──────────────────────────────┘
        │                      ↓
        │          ┌──────────────────────────────┐
        │          │ Plugin returns ">"           │
        │          └──────────┬───────────────────┘
        │                     ↓
        │        ┌──────────────────────────────┐
        ├───────→│ Does trigger match input[0]? │
        │        └──────────┬───────────────────┘
        │                   ↓
        │ NO    ┌──────────────────┐
        │       │ Try next plugin  │
        │       └────────────────────┘
        ↓
     MATCH! ──→ ┌──────────────────────────────────┐
               │ Call handle(">hello world")       │
               └────────────┬─────────────────────┘
                            ↓
               ┌────────────────────────────────────┐
               │ Plugin executes:                   │
               │ - Trim ">" prefix                  │
               │ - Format "You typed: hello world"  │
               │ - Return ActionItem                │
               └────────────┬───────────────────────┘
                            ↓
               ┌───────────────────────────────────┐
               │ Plugin returns:                   │
               │ [{                                │
               │   name: "You typed: hello world", │
               │   exec: "",                       │
               │   keywords: ">"                   │
               │ }]                                │
               └────────────┬──────────────────────┘
                            ↓
               ┌────────────────────────────────────┐
               │ Swift displays result              │
               │ in the launcher UI                 │
               └────────────────────────────────────┘
```

### Memory Diagram

```
┌────────────────────────────────────────────┐
│      Swift Launcher Process Memory         │
│  (Can access filesystem, network, etc.)    │
└────────────────────────────────────────────┘
                    ↓
            ┌─────────────────┐
            │ Wasmtime Store  │
            └─────────────────┘
                    ↓
    ┌───────────────┬───────────────┬────────────────┐
    ↓               ↓               ↓                ↓
┌─────────┐    ┌─────────┐    ┌──────────┐      ┌────────────┐
│echo.wasm│    │calc.wasm│    │ dir.wasm │      │custom.wasm │
│────────────────────────────────────────────────────────────│
│ ISOLATED SANDBOX: No direct filesystem access              │
│ Can only call functions in linker                          │
└────────────────────────────────────────────────────────────┘
```

Each plugin runs in its own sandbox:
- Can access its own linear memory (4GB max by default)
- Can call functions registered by the linker (WASI, host functions)
- Cannot access Swift Launcher's memory
- Cannot access filesystem
- Cannot access network

---

## How Information Flows

### Function Call Example

**Step 1: Host prepares arguments**
```rust
// In Swift Launcher
let input = ">hello world".to_string();
let results = guest.handle(&mut store, &input)?;
```

**Step 2: Component Model marshals data**
```
String "hello world" → Serialized to WASM linear memory at offset 0x1000
Pointer 0x1000 & Length 11 → Passed to plugin as (i32, i32)
```

**Step 3: Plugin function executes**
```rust
// In echo.wasm
fn handle(input: String) -> Vec<ActionItem> {
    // input is deserialized from pointer/length
    let cleaned = input.trim_start_matches('>').trim();
    // ... build result ...
}
```

**Step 4: Plugin returns data**
```
Vec[ActionItem {...}] → Serialized to linear memory
Pointer 0x2000 → Returned to host
```

**Step 5: Host deserializes result**
```rust
// In Swift Launcher
let results: Vec<ActionItem> = ...; // Deserialized from plugin's memory
println!("Result: {:?}", results[0].name);
```

### The Barrier

```
┌────────────────────┐
│ Swift Launcher     │
│                    │
│ String "hello"     │
│ Vec<Item>          │
└────────┬───────────┘
         │ Must serialize/deserialize
         │ at the boundary
         ↓
    ╔════════════╗
    ║ BOUNDARY   ║
    ║ (Component ║
    ║ Model)     ║
    ╚════════════╝
         ↓
┌────────┬────────────┐
│ echo.wasm (Plugin)  │
│                     │
│ Serialized data     │
│ (pointers, lengths) │
└─────────────────────┘
```

The Component Model automatically handles the serialization/deserialization. You just write normal code in your language.

---

## Memory and Isolation

### Linear Memory

WASM uses **linear memory** - a single flat array of bytes (like a C array):

```
WASM Linear Memory (in plugin)
┌─────────┬─────────┬──────────┬─────────┬──────────┐
│ Stack   │ Heap    │ String   │ Array   │ Unused   │
│ 0x00000 │ 0x10000 │ 0x20000  │ 0x30000 │ 0x40000  │
└─────────┴─────────┴──────────┴─────────┴──────────┘
← Grows →                                 ← May grow →
```

Each plugin gets its own **4GB virtual address space** (by default), isolated from other plugins.

### Isolation Benefits

```
echo.wasm crashes     calc.wasm still works
     ↓                        ↓
Sandbox trapped       Sandbox trapped
     ↓                        ↓
No impact on Swift    No impact on Swift
     ↓                        ↓
User restarts plugin  User continues
```

Compare to traditional plugins:
```
One .so crashes → Takes down entire Swift Launcher → User loses work 😞
```

### Memory Safety

WASM ensures memory safety at the binary level:

- No buffer overflows - bounds checking built-in
- No use-after-free - garbage collection or borrow checker
- No null pointer dereferences - type system validates
- No invalid memory access - all offsets verified

This is enforced by the WASM runtime, not by your code. Even if you write unsafe code in C, the runtime checks it.

---

## WASI and System Access

### What is WASI?

**WebAssembly System Interface (WASI)** is a POSIX-like API for WASM:

```
Traditional System Calls
open(), read(), write(), etc.
        ↓
    Kernel
        
WASI Calls
wasi::fs::open(), wasi::fs::read(), etc.
        ↓
Host (Swift Launcher) decides what to allow
```

### WASI Preview 2

Swift Launcher uses **WASI Preview 2** (the current standard):

```rust
// In plugins/component.rs
wasmtime_wasi::p2::add_to_linker_sync(&mut linker)?;
```

This registers ~50 WASI functions, giving plugins controlled access:

**Granted:**
- Basic I/O (stdout for debugging)
- Environment variables
- Current working directory
- Randomness (for UUIDs, etc.)

**Denied:**
- Filesystem access
- Network access
- Process spawning
- Direct clock access

### Custom Host Functions

Swift Launcher can also expose its own functions:

```rust
// Host (Swift Launcher) exposes a custom function
linker.define("swift", "launcher", "get-user-config", |mut caller, arg| {
    let config = swift_config::load();
    Ok(config.serialize())
})?;

// Plugin calls it
let config = host::get_user_config()?;
```

This allows plugins to request data from Swift Launcher safely.

---

## Putting It All Together

### Complete Execution Flow

```
1. User types ">hello"
                ↓
2. Swift calls discover_plugins()
   └─→ Finds ~/.config/swift/plugins/echo.wasm
                ↓
3. For each plugin:
   3a. Load Component from .wasm file
   3b. Create Linker with WASI functions
   3c. Instantiate in Wasmtime Store
   3d. Call get_trigger()
                ↓
4. Trigger matches! (">")
   └─→ Call handle(">hello")
                ↓
5. Plugin executes in sandbox:
   ├─ Trim prefix: "hello"
   ├─ Format result: "You typed: hello"
   └─ Return ActionItem
                ↓
6. Host deserializes result
                ↓
7. UI displays: "You typed: hello"
```

### Key Guarantees

**Isolation** - Plugin can't escape its sandbox
**Safety** - WASM runtime enforces bounds checking
**Performance** - Near-native execution (5-10% overhead)
**Portability** - Same binary on any OS
**Simplicity** - Write plugins in any language
**Reliability** - One plugin crash doesn't affect others

---

## Why This Architecture Wins

### The Old Way (Dynamic Libraries)
```
echo.so (for Linux)
echo.dll (for Windows)
echo.dylib (for macOS)

User downloads wrong binary → Crash
```

### The New Way (WASM Components)
```
echo.wasm (works everywhere)

User downloads once → Works on Linux, macOS, Windows, ARM
```

### Performance Comparison

| Architecture | Startup Time | Memory | Speed | Crash Impact |
|:---|:---|:---|:---|:---|
| Native .so | <1ms | ~100KB | 100% | Crashes host |
| Python scripts | 100ms+ | 50MB+ | 10% | Crashes host |
| WASM component | ~5ms | ~1MB | 95% | Isolated |

WASM is the sweet spot: **fast enough** ✓, **safe enough** ✓, **portable enough** ✓

---

## Further Reading

- [WebAssembly.org](https://webassembly.org/)
- [Component Model Spec](https://component-model.bytecodealliance.org/)
- [WASI Preview 2](https://github.com/WebAssembly/WASI)
- [Wasmtime Book](https://docs.wasmtime.dev/)
