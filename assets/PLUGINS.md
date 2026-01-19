# Creating Swift Launcher Plugins for WASM -- PLUGIN TESTING NOT DONE YET!

## Table of Contents
1. [Quick Start](#quick-start)
2. [The Echo Plugin (All Languages)](#the-echo-plugin-all-languages)
3. [Building for wasm32-wasip2](#building-for-wasm32-wasip2)
4. [Testing Your Plugin](#testing-your-plugin)
5. [Best Practices](#best-practices)
6. [Troubleshooting](#troubleshooting)

---

## Quick Start

#### Quick Start

**Example: Echo Plugin in Rust**

1. Install tools:
```bash
cargo install cargo-component
rustup target add wasm32-wasip2
```

2. Create project:
```bash
cargo component new --lib echo-plugin
cd echo-plugin
```

3. Implement the plugin (see PLUGINS.md for full code):
```rust
impl Guest for Echo {
    fn get_trigger() -> String {
        ">".to_string()
    }

    fn handle(input: String) -> Vec<ActionItem> {
        let cleaned = input.trim_start_matches('>').trim();
        vec![ActionItem {
            name: format!("You typed: {}", cleaned),
            exec: String::new(),
            keywords: ">".to_string(),
        }]
    }
}
```

4. Build and install:
```bash
cargo component build --release
cp target/wasm32-wasip2/release/echo_plugin.wasm ~/.config/swift/plugins/
```

5. Restart Shift and type `>hello world` - your plugin is live!

### Installing Plugins

Simply drop compiled `.wasm` files into:
```
~/.config/swift/plugins/
```

Shift automatically discovers and loads them on startup. No restart required (in future versions with hot-reload).

### Plugin Interface

Every plugin must implement two functions (defined in `plugin.wit`):

```wit
get-trigger() -> string           # Returns the trigger character (e.g., ">")
handle(input: string) -> list<action-item>  # Processes input, returns results
```

The `action-item` struct contains:
- `name`: Display string shown in launcher
- `exec`: Shell command to execute when selected (can be empty)
- `keywords`: Used for fuzzy search ranking

---

## The Echo Plugin (All Languages)

The echo plugin is trivial but demonstrates the full pattern. When you type `>hello world`, it displays `You typed: hello world`.

### Rust

#### Installation
```bash
cargo install cargo-component
rustup target add wasm32-wasip2
```

#### Create Project
```bash
cargo component new --lib echo-plugin
cd echo-plugin
```

#### Cargo.toml
```toml
[package]
name = "echo-plugin"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]

[dependencies]
# No external dependencies needed for echo, but you can add any Rust crate

[package.metadata.component]
package = "swift:launcher"

[package.metadata.component.target]
path = "../../plugin.wit"
world = "plugin-world"
```

#### src/lib.rs
```rust
use bindings::exports::swift::launcher::runner::{ActionItem, Guest};
mod bindings;

struct Echo;

impl Guest for Echo {
    fn get_trigger() -> String {
        ">".to_string()
    }

    fn handle(input: String) -> Vec<ActionItem> {
        let cleaned = input.trim_start_matches('>').trim();
        vec![ActionItem {
            name: format!("You typed: {}", cleaned),
            exec: String::new(),
            keywords: ">".to_string(),
        }]
    }
}

bindings::export!(Echo with_types_in bindings);
```

#### Build & Install
```bash
cargo component build --release
cp target/wasm32-wasip2/release/echo_plugin.wasm ~/.config/swift/plugins/
```

---

### Python

#### Installation
```bash
pip install componentize-py
```

#### Generate Bindings
```bash
# Copy plugin.wit to your project directory
componentize-py -d plugin.wit -w plugin-world bindings .
```

#### echo.py
```python
from typing import List
from wit_world import exports
from wit_world.exports import runner

class Runner:
    def get_trigger(self) -> str:
        return ">"
    
    def handle(self, input: str) -> List[runner.ActionItem]:
        cleaned = input.lstrip(">").strip()
        return [runner.ActionItem(
            name=f"You typed: {cleaned}",
            exec="",
            keywords=">"
        )]
```

#### Build & Install
```bash
componentize-py -d plugin.wit -w plugin-world componentize echo -o echo.wasm
mkdir -p ~/.config/swift/plugins
cp echo.wasm ~/.config/swift/plugins/
```

**Note:** Python components are larger but great for prototyping. Use Python when you need rapid iteration or want to integrate pip packages.

---

<!-- ### Go (TinyGo) -->

<!-- #### Installation -->
<!-- ```bash -->
<!-- # Install TinyGo (not standard Go) -->
<!-- # macOS: brew install tinygo -->
<!-- # Linux: https://tinygo.org/getting-started/install/ -->

<!-- go install github.com/bytecodealliance/wit-bindgen-go/cmd/wit-bindgen-go@latest -->
<!-- ``` -->

<!-- #### Generate Bindings -->
<!-- ```bash -->
<!-- wit-bindgen-go generate plugin.wit -->
<!-- ``` -->

<!-- #### echo.go -->
<!-- ```go -->
<!-- package main -->

<!-- import ( -->
    <!-- "strings" -->
    <!-- gen "gen"  // Generated bindings package -->
<!-- ) -->

<!-- type EchoPlugin struct{} -->

<!-- func (e EchoPlugin) GetTrigger() string { -->
    <!-- return ">" -->
<!-- } -->

<!-- func (e EchoPlugin) Handle(input string) []gen.ActionItem { -->
    <!-- cleaned := strings.TrimPrefix(strings.TrimSpace(input), ">") -->
    <!-- cleaned = strings.TrimSpace(cleaned) -->
    
    <!-- return []gen.ActionItem{ -->
        <!-- { -->
            <!-- Name:     "You typed: " + cleaned, -->
            <!-- Exec:     "", -->
            <!-- Keywords: ">", -->
        <!-- }, -->
    <!-- } -->
<!-- } -->

<!-- func init() { -->
    <!-- gen.SetExportsSoftwareLauncherRunner(EchoPlugin{}) -->
<!-- } -->

<!-- func main() {} -->
<!-- ``` -->

<!-- #### Build & Install -->
<!-- ```bash -->
<!-- tinygo build -target=wasip2 -o echo.wasm echo.go -->
<!-- cp echo.wasm ~/.config/swift/plugins/ -->
<!-- ``` -->

<!-- **Note:** Use TinyGo (not standard Go). Standard Go runtime is too heavy for WASM. -->

---

### JavaScript (Node.js/Deno)

#### Installation
```bash
npm install -g @bytecodealliance/jco @bytecodealliance/componentize-js @bytecodealliance/preview2-shim
```

#### echo.js
```javascript
export const runner = {
  getTrigger() {
    return ">";
  },
  
  handle(input) {
    const cleaned = input.replace(/^>\s*/, '').trim();
    return [{
      name: `You typed: ${cleaned}`,
      exec: "",
      keywords: ">"
    }];
  }
};
```

#### Build & Install
```bash
jco componentize echo.js --wit plugin.wit -n plugin-world -o echo.wasm
cp echo.wasm ~/.config/swift/plugins/
```

**Note:** Experimental tooling. Great if you want to reuse npm packages. Can import npm modules in your plugin code. Also slow like Python.

---

### C/C++ -- Haven't tested yet!

#### Installation
```bash
# Download WASI SDK from:
# https://github.com/WebAssembly/wasi-sdk/releases
export WASI_SDK_PATH=/path/to/wasi-sdk

# Install wit-bindgen for C
cargo install wit-bindgen-cli
```

#### Generate Bindings
```bash
wit-bindgen-cli c plugin.wit --out-dir .
```

#### echo.c
```c
#include "plugin.h"
#include <string.h>
#include <stdio.h>
#include <stdlib.h>

void swift_launcher_runner_get_trigger(swift_launcher_runner_string_t *ret) {
    swift_launcher_runner_string_set(ret, ">");
}

void swift_launcher_runner_handle(
    swift_launcher_runner_string_t *input,
    swift_launcher_runner_list_action_item_t *ret
) {
    // Skip the '>' prefix
    const char *ptr = input->ptr;
    size_t len = input->len;
    
    while (len > 0 && (*ptr == '>' || *ptr == ' ')) {
        ptr++;
        len--;
    }
    
    // Allocate one result item
    ret->len = 1;
    ret->ptr = malloc(sizeof(swift_launcher_runner_action_item_t));
    
    // Build result string
    char buffer[512];
    snprintf(buffer, sizeof(buffer), "You typed: %.*s", (int)len, ptr);
    
    swift_launcher_runner_string_set(&ret->ptr[0].name, buffer);
    swift_launcher_runner_string_set(&ret->ptr[0].exec, "");
    swift_launcher_runner_string_set(&ret->ptr[0].keywords, ">");
}
```

#### Build & Install
```bash
# Compile to WASM
$WASI_SDK_PATH/bin/clang \
  -target wasm32-wasip2 \
  -O2 \
  -mexec-model=reactor \
  -o echo-core.wasm \
  echo.c plugin_component_type.o

# Convert to component
wasm-tools component new echo-core.wasm -o echo.wasm
cp echo.wasm ~/.config/swift/plugins/
```

---

<!-- ### Zig (Modern Systems Language) -->

<!-- #### Installation -->
<!-- ```bash -->
<!-- # Install Zig from https://ziglang.org/download -->
<!-- zig --version  # Should be 0.13+ -->

<!-- # Install wit-bindgen for Zig -->
<!-- # (Still in early development) -->
<!-- ``` -->

<!-- #### echo.zig -->
<!-- ```zig -->
<!-- const std = @import("std"); -->

<!-- pub fn getTrigger() ![]u8 { -->
    <!-- return ">"; -->
<!-- } -->

<!-- pub fn handle(input: []u8) ![]u8 { -->
    <!-- var gpa = std.heap.GeneralPurposeAllocator(.{}){}; -->
    <!-- defer _ = gpa.deinit(); -->
    <!-- const allocator = gpa.allocator(); -->
    
    <!-- const cleaned = std.mem.trim(u8, input, "> \t\n"); -->
    <!-- const result = try std.fmt.allocPrint(allocator, "You typed: {s}", .{cleaned}); -->
    
    <!-- return result; -->
<!-- } -->
<!-- ``` -->

<!-- #### Build & Install -->
<!-- ```bash -->
<!-- zig build-exe echo.zig -target wasm32-wasi -O ReleaseSmall -fno-entry -rdynamic --export-table --name echo-core -->
<!-- wasm-tools component embed plugin.wit echo-core.wasm -o echo-embeded.wasm -->
<!-- wget https://github.com/bytecodealliance/wasmtime/releases/tag/v40.0.2/wasi_snapshot_preview1.reactor.wasm -->
<!-- wasm-tools component new echo.wasm -o echo-component.wasm -->
<!-- cp echo-component.wasm ~/.config/swift/plugins/ -->
<!-- ``` -->

<!-- **Note:** Zig tooling for WASM components is still developing. -->

---

### Verifying Your Build

After compilation, verify it's a valid Component:

```bash
wasm-tools component info echo.wasm
# Should show:
# Package: swift:launcher
# Exports: swift:launcher/runner
```

---

## Testing Your Plugin

### Manual Testing in Swift Launcher

1. **Build your plugin** (using language-specific instructions above)
2. **Copy to config directory:**
   ```bash
   cp echo.wasm ~/.config/swift/plugins/
   ```
3. **Restart Swift Launcher** (or reload plugins)
4. **Type the trigger** in the search bar (e.g., `>hello world`)
5. **Verify the result** appears in the list

### Debugging

Enable Rust backtrace for more error details:

```bash
RUST_BACKTRACE=1 cargo run --release
```

Look for errors like:
- `NoWaylandLib` - missing Wayland (check flake.nix)
- `component imports instance...` - missing WASI bindings
- `cannot find function` - incorrect binding imports

### Testing Individual Plugins Offline

Use `wasmtime` CLI:

```bash
wasmtime echo.wasm  # Runs the component directly
```

---

## Best Practices

### 1. Keep Plugins Focused
Each plugin should handle **one trigger** and do **one thing well**.

❌ Bad: A single plugin handling `=`, `/`, and `$`
✅ Good: Three separate plugins

### 2. Handle Errors Gracefully
Always return *something*, even on error:

```rust
fn handle(input: String) -> Vec<ActionItem> {
    match some_fallible_operation() {
        Ok(result) => vec![ActionItem { name: result, ... }],
        Err(e) => vec![ActionItem { 
            name: format!("Error: {}", e),  // Still return a result
            exec: String::new(),
            keywords: trigger,
        }],
    }
}
```

### 3. Minimize Binary Size
- Rust: Use `strip = "symbols"` in release profile
- Python: Remove unnecessary dependencies
- Go: Use TinyGo instead of standard Go
- C: Compile with `-O2` optimization

### 4. Test with Multiple Character Inputs
Plugins receive the full input string including the trigger:

```
User types: "=2+2"
Plugin receives: "=2+2"  ← Your code should trim the trigger
```

### 5. Use Consistent Naming
Trigger character + action, e.g:
- `=` for calculator
- `/` for directory
- `$` for currency
- `!` for system commands
- `@` for web search

### 6. Document Your Plugin
Include a README in your plugin repo:
```
# Echo Plugin

Trigger: `>`

Usage: Type `>message` to echo the message back.

Example: `>hello world` → displays "You typed: hello world"
```

---

## Troubleshooting

### "cannot find bindings module"

**Cause:** The Component Model bindings haven't been generated.

**Solution:**
```bash
# Ensure plugin.wit is in the correct location
cargo component build  # This auto-generates bindings
```

### "method has incompatible type for trait"

**Cause:** Your function signature doesn't match the .wit definition.

**Solution:** Check that return type matches exactly:
```wit
handle: func(input: string) -> list<action-item>;  # Returns Vec, not Option
```

### "error: component imports instance..., but a matching implementation was not found"

**Cause:** The host's Linker doesn't have all required WASI functions registered.

**Solution:** In `plugins.rs`, ensure:
```rust
wasmtime_wasi::p2::add_to_linker_sync(&mut linker)?;
config.async_support(true);  // Required for Preview 2
```

### "Binary is too large"

**Cause:** Dependencies bloated the binary.

**Solution:**
- Rust: Use `lto = "fat"` and `strip = "symbols"`
- Python: Remove unused packages before componentizing
- Go: Make sure you're using TinyGo, not standard Go

### "Plugin crashes silently"

**Cause:** Unhandled panic in plugin code.

**Solution:**
```bash
RUST_BACKTRACE=full ./swift-launcher
```

The backtraces will show exactly where in your plugin code it crashed.

