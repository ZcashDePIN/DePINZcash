# Contributing to DePINZcash

Thank you for your interest in contributing to DePINZcash! This document provides guidelines for contributing to the project.

## Ways to Contribute

### 1. Code Contributions
- Bug fixes
- New features
- Performance improvements
- Test coverage

### 2. Documentation
- Improve existing docs
- Add tutorials
- Translate documentation
- Fix typos

### 3. Testing
- Run beta versions
- Report bugs
- Verify fixes
- Test on different platforms

### 4. Community
- Answer questions on Discord
- Help new users
- Write blog posts
- Create video tutorials

## Development Setup

### Prerequisites
- Rust 1.70+
- Git
- Zebra full node (for testing)

### Getting Started

```bash
# Fork the repository on GitHub

# Clone your fork
git clone https://github.com/YOUR_USERNAME/DePINZcash
cd DePINZcash

# Add upstream remote
git remote add upstream https://github.com/depinzcash/DePINZcash

# Install dependencies
./scripts/setup.sh

# Build the project
cd prover
cargo build

# Run tests
cargo test
```

## Code Style

### Rust
We follow the official Rust style guide:

```bash
# Format code
cargo fmt

# Lint
cargo clippy

# Before committing
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

### Commit Messages

Use conventional commits format:

```
type(scope): subject

body

footer
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation
- `style`: Formatting
- `refactor`: Code restructuring
- `test`: Adding tests
- `chore`: Maintenance

**Example:**
```
feat(prover): add support for testnet proofs

Implement testnet proof generation alongside mainnet.
Users can now specify --network flag to generate testnet proofs.

Closes #42
```

## Pull Request Process

### 1. Create a Branch

```bash
git checkout -b feature/your-feature-name
```

Use prefixes:
- `feature/` - new features
- `fix/` - bug fixes
- `docs/` - documentation
- `refactor/` - code improvements

### 2. Make Changes

- Write clear, concise code
- Add tests for new functionality
- Update documentation
- Follow existing code style

### 3. Test Your Changes

```bash
# Run all tests
cargo test

# Test the binary
cargo build --release
./target/release/depinzcash-prover --help
```

### 4. Commit

```bash
git add .
git commit -m "feat: add awesome feature"
```

### 5. Push

```bash
git push origin feature/your-feature-name
```

### 6. Open Pull Request

- Go to GitHub and create a PR
- Fill out the PR template
- Link related issues
- Wait for review

### 7. Address Feedback

- Respond to review comments
- Make requested changes
- Push updates to the same branch

## Testing Guidelines

### Unit Tests

Place tests in the same file as the code:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reward_calculation() {
        // Test implementation
    }
}
```

### Integration Tests

Place in `tests/` directory:

```rust
// tests/proof_generation.rs

#[test]
fn test_full_proof_flow() {
    // End-to-end test
}
```

### Running Tests

```bash
# All tests
cargo test

# Specific test
cargo test test_reward_calculation

# With output
cargo test -- --nocapture

# Documentation tests
cargo test --doc
```

## Documentation

### Code Documentation

Use Rust doc comments:

```rust
/// Generates a zero-knowledge proof of node operation.
///
/// # Arguments
///
/// * `metrics` - Node metrics to prove
///
/// # Returns
///
/// A `Proof` object containing the Halo 2 proof
///
/// # Errors
///
/// Returns error if proof generation fails
pub async fn generate_proof(&self, metrics: &NodeMetrics) -> Result<Proof> {
    // Implementation
}
```

### Documentation Files

- Place in `docs/` directory
- Use Markdown format
- Include code examples
- Keep language clear and concise

## Issue Reporting

### Bug Reports

Include:
- DePINZcash version
- Operating system
- Zebra version
- Steps to reproduce
- Expected vs actual behavior
- Error messages/logs

**Template:**
```markdown
## Bug Description
Brief description of the bug

## Steps to Reproduce
1. Step one
2. Step two
3. ...

## Expected Behavior
What should happen

## Actual Behavior
What actually happens

## Environment
- OS: Ubuntu 22.04
- DePINZcash version: 0.1.0
- Zebra version: 1.5.0
- Rust version: 1.75.0

## Logs
```
paste relevant logs here
```
```

### Feature Requests

Include:
- Use case
- Proposed solution
- Alternative solutions considered
- Willingness to implement

## Code Review

### As a Reviewer

- Be respectful and constructive
- Ask questions instead of making demands
- Explain reasoning behind suggestions
- Approve when ready, request changes if needed

### As an Author

- Don't take feedback personally
- Ask for clarification if needed
- Respond to all comments
- Thank reviewers for their time

## Security Issues

**Do not** open public issues for security vulnerabilities.

Instead:
- Email: security@depinzcash.io
- Use PGP key if available
- Provide detailed description
- Allow time for fix before disclosure

We'll acknowledge within 48 hours and provide a timeline for fixes.

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

## Questions?

- **Discord:** [discord.gg/depinzcash](https://discord.gg/depinzcash)
- **GitHub Discussions:** [github.com/depinzcash/DePINZcash/discussions](https://github.com/depinzcash/DePINZcash/discussions)
- **Email:** dev@depinzcash.io

---

Thank you for contributing to DePINZcash! 🦓⚡
