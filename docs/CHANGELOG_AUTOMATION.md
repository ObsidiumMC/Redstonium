# Automated Changelog System

This project uses an automated changelog generation system that creates and maintains the `CHANGELOG.md` file based on your git commits and releases.

## How It Works

The automated changelog system consists of three GitHub Actions workflows:

### 1. Release Changelog (`release.yml`)
- **Trigger**: When you create a git tag (e.g., `v1.0.0`)
- **Function**: Automatically generates a changelog entry for the new version and updates `CHANGELOG.md`
- **Integration**: Integrated into the release workflow to create meaningful release notes

### 2. Continuous Changelog (`changelog-generator.yml`)
- **Trigger**: On pushes to main branch or manually via workflow dispatch
- **Function**: Continuously updates the changelog with new commits
- **Use Case**: For maintaining an up-to-date changelog between releases

### 3. Release Drafter (`changelog.yml`)
- **Trigger**: On pull requests and pushes
- **Function**: Creates draft releases with auto-generated release notes
- **Use Case**: For preparing releases and reviewing changes

## Commit Message Conventions

The changelog generator works best with conventional commit messages:

```
<type>(<scope>): <description>

[optional body]

[optional footer(s)]
```

### Supported Types

- `feat`: New features → **Added** section
- `fix`: Bug fixes → **Fixed** section  
- `docs`: Documentation → **Changed** section
- `style`: Code style changes → **Changed** section
- `refactor`: Code refactoring → **Changed** section
- `perf`: Performance improvements → **Changed** section
- `test`: Adding tests → **Added** section
- `chore`: Maintenance tasks → **Changed** section
- `build`: Build system changes → **Changed** section
- `ci`: CI/CD changes → **Changed** section
- `revert`: Reverting changes → **Fixed** section

### Examples

```bash
# Good commit messages
feat(auth): add Microsoft OAuth2 integration
fix(launcher): resolve Java detection on macOS
docs: update installation instructions
chore(deps): update dependencies to latest versions

# These will be automatically categorized in the changelog
```

## Manual Usage

### Generate Changelog for Current Version
```bash
# Trigger the workflow manually
gh workflow run changelog-generator.yml

# Or trigger with a specific version
gh workflow run changelog-generator.yml -f version=1.2.3
```

### Create a Release with Auto-Generated Changelog
```bash
# Create and push a tag - this will automatically update the changelog
git tag v1.0.0
git push origin v1.0.0

# The release workflow will:
# 1. Generate changelog entry for v1.0.0
# 2. Update CHANGELOG.md
# 3. Create GitHub release with the changelog content
# 4. Build and upload release assets
```

## Customization

### Modifying Categories
Edit `.github/release-drafter.yml` to customize how commits are categorized:

```yaml
categories:
  - title: '🚀 New Features'
    labels: ['feature', 'enhancement']
  - title: '🐛 Bug Fixes'  
    labels: ['fix', 'bugfix', 'bug']
```

### Excluding Commits
Commits are automatically excluded if they:
- Start with "Merge" (merge commits)
- Contain "CHANGELOG" in the message
- Contain "[skip ci]" in the message

### Manual Changelog Entries
You can still manually edit `CHANGELOG.md`. The automated system will:
- Preserve existing entries
- Add new entries at the top
- Maintain the Keep a Changelog format

## File Structure

```
.github/
├── workflows/
│   ├── release.yml              # Main release workflow with changelog
│   ├── changelog-generator.yml  # Standalone changelog generator
│   └── changelog.yml           # Release drafter integration
└── release-drafter.yml         # Release drafter configuration
```

## Best Practices

1. **Use Conventional Commits**: Follow the commit message convention for best results
2. **Review Before Release**: Check the generated changelog before releasing
3. **Meaningful Commits**: Write descriptive commit messages - they become your changelog
4. **Scope Usage**: Use scopes to organize changes by component (e.g., `feat(auth):`, `fix(ui):`)
5. **Breaking Changes**: Use `BREAKING CHANGE:` in the commit footer for major version bumps

## Troubleshooting

### Changelog Not Updating
- Check if commits follow conventional commit format
- Ensure commits don't contain "[skip ci]"
- Verify GitHub Actions have write permissions

### Missing Entries
- Check if commits were made after the last tag
- Verify commit messages don't start with "Merge"
- Look at the Actions logs for any errors

### Format Issues
- The system maintains Keep a Changelog format
- Manual edits are preserved
- Contact maintainers if you notice formatting problems

## Migration from Manual Changelog

If you have an existing manual `CHANGELOG.md`:

1. The automated system will preserve existing entries
2. New entries will be added at the top
3. Consider reviewing and reformatting old entries to match the new style
4. The first automated run will create a new version entry based on recent commits

---

This automated system ensures your changelog stays up-to-date with minimal manual effort while maintaining professional formatting and comprehensive change tracking.
