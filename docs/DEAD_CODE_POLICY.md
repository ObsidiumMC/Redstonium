# Dead Code Policy

This document explains Rustified's strict no-dead-code policy and how it's enforced.

## 🚫 What is Dead Code?

Dead code refers to any code that is written but never used or executed. In Rust, this includes:

- **Unused functions** - Functions that are defined but never called
- **Unused variables** - Variables that are declared but never read
- **Unused imports** - `use` statements that import items never used
- **Unused struct fields** - Fields in structs that are never accessed
- **Unused enum variants** - Enum variants that are never constructed or matched
- **Unused constants** - Constants that are defined but never referenced
- **Unused macros** - Macros that are defined but never invoked
- **Unreachable code** - Code that can never be executed due to control flow

## ⚠️ Our Policy

**Rustified maintains a zero-tolerance policy for dead code.**

### Why This Policy Exists

1. **Code Quality** - Dead code clutters the codebase and makes it harder to understand
2. **Maintenance Burden** - Unused code still needs to be maintained and updated
3. **Security Risk** - Dead code can contain vulnerabilities that aren't obvious
4. **Performance** - Dead code increases compile times and binary size
5. **Developer Confusion** - Future contributors may waste time trying to understand unused code
6. **Technical Debt** - Accumulating dead code creates technical debt

### What Gets Rejected

Our CI/CD pipeline will **automatically reject** any Pull Request that contains:

- Any `warning: dead_code` messages
- Any `warning: unused_*` messages (variables, imports, functions, etc.)
- Any code that doesn't serve a clear purpose
- Commented-out code blocks
- Experimental code that isn't actively used

## 🔍 How We Detect Dead Code

### Automated Checks

1. **Compilation Warnings** - `cargo check` with dead code warnings promoted to errors
2. **Clippy Lints** - Comprehensive clippy checks with strict dead code rules
3. **CI/CD Pipeline** - Dedicated workflow that runs on every PR
4. **Pre-commit Hooks** - Local hooks that catch issues before committing

### Tools and Commands

```bash
# Basic dead code check
cargo check --all-targets

# Comprehensive clippy check
cargo clippy --all-targets --all-features -- \
  -D dead_code \
  -D unused_imports \
  -D unused_variables \
  -D unused_mut \
  -D warnings

# Using environment variables
RUSTFLAGS="-D dead_code -D unused_imports -D unused_variables" cargo check

# Using justfile (if installed)
just dead-code-check
just check-all
```

## ✅ Exceptions and Best Practices

### Legitimate Exceptions

Some code may appear "unused" but serves important purposes:

#### Test Code
```rust
#[cfg(test)]
mod tests {
    use super::*;  // This is OK - test-only code
    
    #[test]
    fn test_something() {
        // Test code is exempt from dead code rules
    }
}
```

#### Feature-Gated Code
```rust
#[cfg(feature = "experimental")]
pub fn experimental_feature() {
    // OK if the feature is documented and intentional
}
```

#### Debug/Development Code
```rust
#[cfg(debug_assertions)]
fn debug_helper() {
    // OK for debug builds only
}
```

#### Platform-Specific Code
```rust
#[cfg(target_os = "windows")]
fn windows_specific() {
    // OK for platform-specific functionality
}
```

#### Trait Implementations
```rust
// Sometimes trait implementations require unused parameters
impl SomeTrait for MyStruct {
    fn method(&self, _unused_param: i32) -> bool {
        // The underscore prefix tells Rust this is intentionally unused
        true
    }
}
```

### Best Practices

1. **Use underscore prefixes** for intentionally unused parameters: `_param`
2. **Use `#[allow(dead_code)]`** sparingly and only when necessary
3. **Remove commented-out code** - use git history instead
4. **Extract experimental code** to separate branches
5. **Use feature flags** for optional functionality
6. **Document exceptions** clearly in code comments

## 🛠️ Developer Workflow

### Before Writing Code

1. **Understand the requirements** - Only write code that serves a clear purpose
2. **Plan your implementation** - Avoid speculative or "just in case" code
3. **Consider the scope** - Will this code be used immediately?

### During Development

1. **Run checks frequently** - Don't let dead code accumulate
2. **Use the pre-commit hooks** - Let them catch issues early
3. **Review your own code** - Remove unused imports and variables as you go

```bash
# Quick check during development
cargo check --all-targets
```

### Before Submitting PR

1. **Run comprehensive checks**:
   ```bash
   just check-all  # or
   cargo clippy --all-targets -- -D dead_code -D warnings
   cargo test
   ```

2. **Review the diff** - Look for any unused imports or variables
3. **Test your changes** - Ensure everything you added is actually used

### If You Get Dead Code Warnings

1. **Don't ignore them** - Fix them instead
2. **Remove unused code** - Don't try to work around the warnings
3. **Ask for help** - If you're unsure whether code is needed

## 🤖 CI/CD Enforcement

### What Happens When Dead Code is Detected

1. **Build fails** - The CI pipeline will fail with clear error messages
2. **PR is blocked** - GitHub will prevent merging until issues are fixed
3. **Clear feedback** - Error messages will show exactly what needs to be removed

### Example CI Output

```
❌ Dead code detected! The following warnings were found:
warning: function `unused_helper` is never used
  --> src/example.rs:42:4
   |
42 | fn unused_helper() {
   |    ^^^^^^^^^^^^^

Please remove all unused code before submitting your PR.
Refer to CONTRIBUTING.md for our dead code policy.
```

## 🔧 Troubleshooting

### Common Issues and Solutions

#### "But I need this code for later!"
- Create a separate branch for future work
- File an issue to track the planned feature
- Use feature flags if the code is part of a larger feature

#### "This code is used, but Rust doesn't see it!"
- Check if you're missing `pub` keywords for public APIs
- Ensure the code is actually called from somewhere
- Use `#[allow(dead_code)]` with a comment explaining why

#### "The warning is in generated code"
- Use `#[allow(dead_code)]` on the generated items
- Configure the code generator to avoid unused code
- Consider if the generation is necessary

#### "This is a trait method that's intentionally unused"
- Use underscore prefix: `fn method(&self, _param: Type)`
- Add `#[allow(unused_variables)]` to the specific method

### Getting Help

If you're stuck with dead code warnings:

1. **Check this documentation** first
2. **Look at similar code** in the codebase
3. **Ask on GitHub Discussions** with specific examples
4. **Mention the issue** in your PR for reviewer guidance

## 📚 Additional Resources

- [Rust Book - Controlling Warnings](https://doc.rust-lang.org/book/ch03-05-control-flow.html)
- [Clippy Lint Documentation](https://rust-lang.github.io/rust-clippy/master/index.html)
- [Rust Compiler Warning Index](https://doc.rust-lang.org/rustc/lints/index.html)
- [Project Contributing Guidelines](CONTRIBUTING.md)

---

Remember: **Every line of code is a liability**. Code that doesn't serve a purpose shouldn't exist in our codebase. This policy helps us maintain a high-quality, maintainable project that's easy for new contributors to understand and work with.
