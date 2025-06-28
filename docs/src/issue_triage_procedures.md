# Issue Triage Procedures

This document outlines the procedures for triaging and managing issues in the μNet repository.

## Triage Process Overview

### 1. Initial Assessment (Within 24 hours)

When a new issue is created:

1. **Label Assignment**: Add appropriate labels based on issue type and content
2. **Priority Assessment**: Evaluate urgency and impact
3. **Component Identification**: Identify which μNet component is affected
4. **Completeness Check**: Ensure all required information is provided

### 2. Label System

#### Issue Types

- `bug` - Confirmed bugs or unexpected behavior
- `enhancement` - New features or improvements
- `documentation` - Documentation issues or improvements
- `question` - Support questions or clarifications
- `security` - Security-related issues (handle with care)

#### Priority Levels

- `priority/critical` - System crashes, data loss, security vulnerabilities
- `priority/high` - Significant functionality broken, major performance issues
- `priority/medium` - Feature requests, minor bugs, improvements
- `priority/low` - Nice-to-have features, cosmetic issues

#### Component Labels

- `component/cli` - μNet CLI interface
- `component/server` - μNet server and API
- `component/core` - Core μNet library
- `component/policy` - Policy engine
- `component/template` - Template system
- `component/git` - Git integration
- `component/database` - Database and migrations
- `component/docs` - Documentation

#### Status Labels

- `triage` - Needs initial assessment
- `needs-info` - Waiting for more information
- `ready-for-work` - Ready to be implemented
- `in-progress` - Currently being worked on
- `needs-review` - Ready for review
- `blocked` - Blocked by dependencies

#### Special Labels

- `good-first-issue` - Suitable for new contributors
- `help-wanted` - Community contributions welcome
- `duplicate` - Duplicate of existing issue
- `wontfix` - Issue will not be addressed
- `invalid` - Invalid issue (spam, not applicable)

### 3. Triage Workflow

#### New Issue Checklist

- [ ] Read issue title and description completely
- [ ] Verify issue template was used and all sections completed
- [ ] Add component label(s)
- [ ] Add priority label
- [ ] Add any relevant status labels
- [ ] Assign to appropriate team member if urgent
- [ ] Remove `triage` label once initial assessment is complete

#### Bug Reports

1. **Reproduce**: Attempt to reproduce the issue
2. **Validate**: Confirm it's actually a bug vs expected behavior
3. **Assess Impact**: Determine how many users are affected
4. **Prioritize**: Set priority based on severity and impact
5. **Label**: Add `component/` and `priority/` labels
6. **Assign**: Assign to appropriate maintainer if high/critical priority

#### Feature Requests

1. **Evaluate Scope**: Assess complexity and effort required
2. **Check Alignment**: Ensure request aligns with project goals
3. **Community Interest**: Check for similar requests or community feedback
4. **Prioritize**: Based on user impact and strategic importance
5. **Label**: Add `enhancement`, `component/`, and `priority/` labels

#### Documentation Issues

1. **Verify Location**: Confirm the documentation location exists
2. **Assess Impact**: Determine how critical the documentation gap is
3. **Complexity**: Evaluate effort required to address
4. **Label**: Add `documentation` and appropriate component labels
5. **Consider**: Mark as `good-first-issue` if suitable for new contributors

#### Questions/Support

1. **Quick Answer**: Provide immediate help if possible
2. **Documentation Gap**: Identify if this reveals a documentation need
3. **Convert**: Consider converting to enhancement if it reveals missing functionality
4. **Close**: Close after providing answer, but encourage follow-up if needed

### 4. Response Time Targets

#### Priority-Based Response Times

- **Critical**: 2 hours during business hours
- **High**: 24 hours
- **Medium**: 72 hours (3 business days)
- **Low**: 1 week

#### Security Issues

- **Immediate**: Acknowledge within 1 hour
- **Assessment**: Complete security assessment within 24 hours
- **Communication**: Provide regular updates to reporter
- **Resolution**: Coordinate with security team for patching

### 5. Common Triage Scenarios

#### Incomplete Bug Reports

```markdown
Thank you for the bug report! To help us investigate, could you please provide:

- [ ] μNet version (`unet --version`)
- [ ] Complete error messages/logs
- [ ] Steps to reproduce the issue
- [ ] Expected vs actual behavior

Please update your issue with this information and we'll take another look.
```

#### Duplicate Issues

```markdown
Thanks for reporting this! This appears to be a duplicate of #XXX. 

I'm closing this issue in favor of the existing one. Please feel free to add any additional information or subscribe to updates on the original issue.
```

#### Feature Requests Needing Clarification

```markdown
Thanks for the feature request! This sounds interesting. Could you help us understand:

- [ ] What specific problem this would solve
- [ ] How you envision using this feature
- [ ] Any alternatives you've considered

This will help us evaluate and potentially implement the feature.
```

#### Mislabeled Issues

- Review and correct labels regularly
- Provide brief explanation when changing labels
- Update priority as more information becomes available

### 6. Escalation Procedures

#### When to Escalate

- Security vulnerabilities
- Critical bugs affecting multiple users
- Issues requiring architectural decisions
- Community conflicts or code of conduct violations

#### Escalation Contacts

- **Security Issues**: <security@your-org.com>
- **Critical Bugs**: Senior maintainers
- **Architecture Decisions**: Core team
- **Community Issues**: Project maintainers

### 7. Quality Assurance

#### Weekly Review

- Review all open issues for stale labels
- Update priorities based on new information
- Close resolved issues that haven't been closed
- Identify issues that need more attention

#### Monthly Analysis

- Analyze triage metrics and response times
- Review label usage and effectiveness
- Identify common issue patterns
- Update triage procedures as needed

### 8. Tools and Automation

#### GitHub Automation

- Use GitHub Actions for automatic labeling based on templates
- Set up stale issue detection and cleanup
- Automate assignment based on component labels

#### Triage Dashboard

- Create saved searches for different triage queues
- Monitor open issues by priority and age
- Track response time metrics

### 9. Best Practices for Triagers

#### Communication

- Be welcoming and professional
- Acknowledge the reporter's effort
- Provide clear next steps
- Ask clarifying questions when needed

#### Decision Making

- When in doubt, err on the side of higher priority
- Consult with team members for unclear cases
- Document reasoning for priority decisions
- Be consistent with labeling across similar issues

#### Time Management

- Use templates for common responses
- Batch similar triage tasks
- Focus on high-priority items first
- Don't let perfect be the enemy of good

## Training and Resources

### New Triager Onboarding

1. Read this document thoroughly
2. Shadow experienced triagers for 1 week
3. Practice on low-priority issues
4. Get familiar with GitHub shortcuts and tools
5. Join triage team communication channels

### Reference Materials

- [μNet Architecture Documentation](01_architecture.md)
- [Contributing Guidelines](../CONTRIBUTING.md)
- [Code of Conduct](../CODE_OF_CONDUCT.md)
- [Security Policy](../SECURITY.md)

---

*This document should be updated regularly based on triage experience and changing project needs.*
