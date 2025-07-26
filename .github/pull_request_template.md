# Pull Request

## Description

Brief description of what this PR accomplishes.

## Type of Change

Please delete options that are not relevant:

- [ ] 🐛 Bug fix (non-breaking change which fixes an issue)
- [ ] ✨ New feature (non-breaking change which adds functionality)
- [ ] 💥 Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [ ] 📚 Documentation update
- [ ] 🎨 Code style/formatting changes
- [ ] ♻️ Refactoring (no functional changes)
- [ ] ⚡ Performance improvements
- [ ] 🧪 Test additions or improvements
- [ ] 🔧 Build system/CI changes

## Testing

- [ ] I have run `cargo test` and all tests pass
- [ ] I have run `cargo clippy` and addressed any warnings
- [ ] I have run `cargo fmt` to ensure consistent formatting
- [ ] I have tested the changes manually (if applicable)
- [ ] I have added tests that prove my fix is effective or that my feature works
- [ ] New and existing unit tests pass locally with my changes

## Embedded/No-Std Compatibility

- [ ] Changes are compatible with `no_std` environments
- [ ] I have tested with `cargo build --no-default-features` (if applicable)
- [ ] I have tested with embedded targets (if applicable)

## Performance Impact

- [ ] No performance impact
- [ ] Minimal performance impact (< 5% change)
- [ ] Significant performance impact (> 5% change) - please explain below

**Performance notes:** (if applicable)

## Documentation

- [ ] I have updated relevant documentation
- [ ] I have added inline comments for complex logic
- [ ] I have updated the README if necessary
- [ ] I have added or updated examples if necessary

## Breaking Changes

If this PR introduces breaking changes, please describe them here and provide migration guidance:

**Breaking changes:** (if applicable)

**Migration guide:** (if applicable)

## Additional Notes

Any additional information, context, or screenshots that reviewers should know about.

## Checklist

- [ ] My code follows the project's style guidelines
- [ ] I have performed a self-review of my own code
- [ ] I have commented my code, particularly in hard-to-understand areas
- [ ] My changes generate no new warnings
- [ ] I have checked my code compiles without warnings on stable Rust
- [ ] I have verified compatibility with the minimum supported Rust version (MSRV)
- [ ] Any dependent changes have been merged and published in downstream modules

---

**For maintainers:**
- [ ] Benchmark results look good (if applicable)
- [ ] Security review completed (if applicable)
- [ ] All CI checks are passing