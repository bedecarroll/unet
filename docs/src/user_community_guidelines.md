<!-- SPDX-License-Identifier: MIT -->

# Community Contribution Guidelines

> **Audience:** Contributors, maintainers, and community members who want to participate in μNet development.  
> **Objective:** Provide clear guidelines for contributing to μNet, building a welcoming community, and maintaining project quality.

---

## Table of Contents

1. [Welcome to the μNet Community](#welcome-to-the-μnet-community)
2. [How to Contribute](#how-to-contribute)
3. [Development Guidelines](#development-guidelines)
4. [Documentation Contributions](#documentation-contributions)
5. [Community Standards](#community-standards)
6. [Support and Help](#support-and-help)
7. [Recognition and Credits](#recognition-and-credits)

---

## Welcome to the μNet Community

### Our Mission

μNet aims to transform network configuration management through modern DevOps practices. We believe that network infrastructure should be:

- **Version controlled** like application code
- **Testable** and verifiable before deployment
- **Consistent** across environments and vendors
- **Accessible** to network engineers of all skill levels

### Community Values

**🤝 Inclusivity**: We welcome contributors from all backgrounds and experience levels  
**📚 Learning**: We prioritize education and knowledge sharing  
**🔧 Quality**: We maintain high standards while being supportive of newcomers  
**🌟 Innovation**: We encourage creative solutions to networking challenges  
**💬 Collaboration**: We value constructive discussion and feedback  

### Ways to Contribute

You don't need to be a Rust expert to contribute! We welcome:

- **Bug reports** and feature requests
- **Documentation** improvements and examples
- **Templates** for different network vendors
- **Policies** for various compliance frameworks
- **Testing** and quality assurance
- **Community support** and answering questions
- **Code contributions** and performance improvements

---

## How to Contribute

### Getting Started

**1. Set Up Your Development Environment**

```bash
# Fork the repository on GitHub
# Clone your fork
git clone https://github.com/your-username/unet.git
cd unet

# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install development tools
rustup component add rustfmt clippy
cargo install cargo-audit cargo-outdated

# Build the project
cargo build

# Run tests to ensure everything works
cargo test
```

**2. Find Something to Work On**

- **Good First Issues**: Look for issues labeled `good-first-issue`
- **Documentation**: Check for `documentation` labeled issues
- **Help Wanted**: Issues labeled `help-wanted` need community assistance
- **Your Own Idea**: Start a discussion in GitHub Discussions

**3. Create Your Contribution**

```bash
# Create a feature branch
git checkout -b feature/your-contribution-name

# Make your changes
# ... develop, test, document ...

# Commit your changes
git add .
git commit -m "feat: add descriptive commit message"

# Push to your fork
git push origin feature/your-contribution-name

# Create a pull request on GitHub
```

### Types of Contributions

#### 🐛 Bug Reports

**Before Reporting:**

- Search existing issues to avoid duplicates
- Test with the latest version of μNet
- Gather relevant system information

**Good Bug Report Template:**

```markdown
**Bug Description**
A clear description of what the bug is.

**To Reproduce**
Steps to reproduce the behavior:
1. Run command '...'
2. With configuration '...'
3. See error

**Expected Behavior**
What you expected to happen.

**Actual Behavior**
What actually happened.

**Environment**
- μNet version: [e.g., v0.5.1]
- OS: [e.g., Ubuntu 22.04]
- Rust version: [e.g., 1.75.0]

**Additional Context**
Configuration files, error logs, screenshots, etc.
```

#### 💡 Feature Requests

**Feature Request Template:**

```markdown
**Feature Description**
A clear description of the feature you'd like to see.

**Use Case**
Explain the problem this feature would solve.

**Proposed Solution**
Your ideas for how this could be implemented.

**Alternative Solutions**
Other approaches you've considered.

**Additional Context**
Examples, mockups, or references to similar features.
```

#### 📝 Documentation Contributions

Documentation is crucial and always needs improvement:

- **Fix typos** and improve clarity
- **Add examples** for complex features
- **Create tutorials** for common use cases
- **Improve API documentation**
- **Translate** documentation to other languages

#### 🔧 Code Contributions

**Code Quality Standards:**

- Follow Rust best practices and idioms
- Add tests for new functionality
- Update documentation for API changes
- Ensure all CI checks pass
- Use descriptive commit messages

---

## Development Guidelines

### Code Style and Standards

**Rust Guidelines:**

```bash
# Format code before committing
cargo fmt

# Check for common mistakes
cargo clippy

# Run all tests
cargo test

# Check for security vulnerabilities
cargo audit

# Check for outdated dependencies
cargo outdated
```

**Commit Message Format:**
We use [Conventional Commits](https://conventionalcommits.org/):

```
type(scope): description

[optional body]

[optional footer]
```

**Types:**

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (no logic changes)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

**Examples:**

```
feat(templates): add Juniper SRX firewall template

fix(policies): handle empty configuration files gracefully

docs(user-guide): add multi-vendor template examples

test(integration): add tests for policy validation
```

### Testing Requirements

**All contributions must include appropriate tests:**

**Unit Tests:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_creation() {
        let node = Node::new("test-router", NodeType::Router);
        assert_eq!(node.name, "test-router");
        assert_eq!(node.node_type, NodeType::Router);
    }

    #[tokio::test]
    async fn test_async_operation() {
        let result = async_function().await;
        assert!(result.is_ok());
    }
}
```

**Integration Tests:**

```rust
// tests/integration_test.rs
use unet::*;

#[tokio::test]
async fn test_end_to_end_workflow() {
    // Test complete workflow from node creation to config generation
    let mut store = setup_test_store().await;
    
    let node = create_test_node(&store).await.unwrap();
    let config = generate_config(&node, "test-template").await.unwrap();
    
    assert!(config.contains("hostname test-node"));
}
```

**Template Tests:**

```yaml
# tests/templates/cisco-router.yaml
template: "cisco/router.j2"
tests:
  - name: "Basic router configuration"
    data:
      node:
        name: "test-router"
        management_ip: "192.168.1.1"
    expected_contains:
      - "hostname test-router"
      - "ip address 192.168.1.1"
```

### Code Review Process

**Pull Request Guidelines:**

1. **Small, Focused Changes**: Keep PRs small and focused on a single feature/fix
2. **Clear Description**: Explain what changed and why
3. **Tests**: Include tests for new functionality
4. **Documentation**: Update relevant documentation
5. **Breaking Changes**: Clearly mark and explain breaking changes

**PR Template:**

```markdown
## Description
Brief description of the changes.

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
- [ ] Unit tests added/updated
- [ ] Integration tests added/updated
- [ ] Manual testing performed

## Checklist
- [ ] Code follows style guidelines
- [ ] Self-review completed
- [ ] Documentation updated
- [ ] Tests pass locally
```

**Review Process:**

1. **Automated Checks**: CI must pass (tests, formatting, clippy)
2. **Code Review**: At least one maintainer review required
3. **Testing**: Reviewer should test the changes
4. **Documentation**: Verify documentation is updated if needed

---

## Documentation Contributions

### Documentation Types

**User Documentation:**

- Getting started guides
- Tutorials and examples
- API reference
- Best practices

**Developer Documentation:**

- Architecture decisions
- Contributing guidelines
- API documentation
- Deployment guides

### Writing Guidelines

**Style Guide:**

- **Clear and Concise**: Use simple, direct language
- **Examples**: Include practical examples
- **Structure**: Use consistent formatting and headers
- **Audience**: Consider the target audience's experience level

**Markdown Standards:**

```markdown
# Main Title (H1)

## Section Title (H2)

### Subsection (H3)

**Bold text** for emphasis
*Italic text* for slight emphasis
`code` for inline code
```

**Code Examples:**

```bash
# Always include comments explaining commands
unet nodes add --name "example-router" --type router

# Show expected output when helpful
# Output: Node 'example-router' created successfully
```

### Documentation Workflow

```bash
# 1. Create documentation branch
git checkout -b docs/your-documentation-topic

# 2. Write/update documentation
# Edit files in docs/src/

# 3. Test documentation build
mdbook serve docs/

# 4. Review in browser at http://localhost:3000

# 5. Commit and push
git add docs/
git commit -m "docs: add user guide for X feature"
git push origin docs/your-documentation-topic

# 6. Create pull request
```

---

## Community Standards

### Code of Conduct

**Our Pledge:**
We pledge to make participation in our community a harassment-free experience for everyone, regardless of:

- Age, body size, disability, ethnicity
- Gender identity and expression
- Level of experience, nationality
- Personal appearance, race, religion
- Sexual identity and orientation

**Expected Behavior:**

- Use welcoming and inclusive language
- Be respectful of differing viewpoints
- Give and accept constructive feedback gracefully
- Focus on what's best for the community
- Show empathy towards community members

**Unacceptable Behavior:**

- Harassment, trolling, or derogatory comments
- Public or private harassment
- Publishing private information without permission
- Any conduct inappropriate in a professional setting

**Enforcement:**
Community violations should be reported to the maintainers. All reports will be:

- Treated confidentially
- Investigated fairly
- Addressed appropriately

### Communication Channels

**GitHub Issues:**

- Bug reports and feature requests
- Technical discussions
- Project planning

**GitHub Discussions:**

- General questions and help
- Community announcements
- Ideas and feedback

**Discord/Slack (if available):**

- Real-time chat and support
- Community events and announcements
- Casual networking discussions

**Mailing List/Newsletter:**

- Major announcements
- Release notifications
- Community highlights

### Community Guidelines

**Be Helpful:**

- Answer questions when you can
- Share your experience and knowledge
- Help newcomers get started

**Be Patient:**

- Remember everyone has different experience levels
- Explain concepts clearly
- Provide context for your suggestions

**Be Constructive:**

- Offer solutions, not just criticism
- Explain the reasoning behind your feedback
- Suggest improvements rather than just pointing out problems

---

## Support and Help

### Getting Help

**Before Asking:**

1. Check the documentation
2. Search existing issues and discussions
3. Try to create a minimal reproducible example

**Where to Ask:**

- **GitHub Discussions**: General questions and help
- **GitHub Issues**: Bug reports and feature requests
- **Community Chat**: Real-time help and discussion

**Asking Good Questions:**

```markdown
**What I'm trying to do:**
Brief description of your goal.

**What I tried:**
Commands/configuration you attempted.

**What happened:**
Error messages, unexpected behavior.

**What I expected:**
What you thought would happen.

**Environment:**
- μNet version
- Operating system
- Relevant configuration
```

### Providing Help

**Help Others by:**

- Answering questions in discussions
- Improving documentation based on common questions
- Creating examples and tutorials
- Testing and reporting bugs

**When Helping:**

- Be patient and respectful
- Ask clarifying questions
- Provide complete answers with context
- Link to relevant documentation

### Resources for Contributors

**Learning Resources:**

- [Rust Book](https://doc.rust-lang.org/book/) - Learn Rust programming
- [Jinja2 Documentation](https://jinja.palletsprojects.com/) - Template engine
- [SeaORM Documentation](https://www.sea-ql.org/SeaORM/) - Database ORM
- [Axum Documentation](https://docs.rs/axum/) - Web framework

**Development Tools:**

- [Rust Analyzer](https://rust-analyzer.github.io/) - VS Code extension
- [Clippy](https://github.com/rust-lang/rust-clippy) - Linting tool
- [Cargo](https://doc.rust-lang.org/cargo/) - Build tool and package manager

---

## Recognition and Credits

### Contributor Recognition

**Ways We Recognize Contributors:**

- **Contributors File**: Listed in CONTRIBUTORS.md
- **Release Notes**: Mentioned in changelog for significant contributions
- **Community Highlights**: Featured in community updates
- **Maintainer Status**: Invited to become maintainer for sustained contributions

### Contribution Types We Value

**Code Contributions:**

- New features and bug fixes
- Performance improvements
- Security enhancements
- Test coverage improvements

**Non-Code Contributions:**

- Documentation improvements
- Community support and moderation
- User experience feedback
- Template and policy contributions
- Translation work

### Becoming a Maintainer

**Maintainer Responsibilities:**

- Review pull requests
- Triage issues
- Help guide project direction
- Mentor new contributors
- Maintain code quality standards

**Path to Maintainership:**

1. **Consistent Contributions**: Regular, quality contributions over time
2. **Community Engagement**: Active participation in discussions and help
3. **Technical Expertise**: Demonstrated understanding of the codebase
4. **Leadership**: Mentoring other contributors
5. **Invitation**: Current maintainers invite qualified contributors

### Special Thanks

We're grateful to all contributors who help make μNet better:

- **Core Contributors**: Those who've made significant code contributions
- **Documentation Heroes**: Contributors who've improved our docs
- **Community Champions**: Active community members who help others
- **Template Creators**: Contributors who've added vendor templates
- **Beta Testers**: Early adopters who helped find and fix issues

---

## Quick Start Checklist

Ready to contribute? Here's your checklist:

- [ ] Read the Code of Conduct
- [ ] Set up development environment
- [ ] Browse open issues for inspiration
- [ ] Join community discussions
- [ ] Make your first contribution (documentation is great!)
- [ ] Submit a pull request
- [ ] Engage with feedback constructively
- [ ] Celebrate your contribution! 🎉

---

## Contact Information

**Project Maintainers:**

- GitHub: [@maintainer-handle](https://github.com/maintainer-handle)
- Email: <maintainers@unet-project.org>

**Community Channels:**

- GitHub Discussions: [μNet Discussions](https://github.com/your-org/unet/discussions)
- Issues: [μNet Issues](https://github.com/your-org/unet/issues)

**Security Issues:**

- Email: <security@unet-project.org>
- Please do not report security issues publicly

---

Thank you for considering contributing to μNet! Every contribution, no matter how small, helps make network configuration management better for everyone. We look forward to working with you! 🚀
