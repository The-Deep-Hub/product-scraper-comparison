# Contributing Guidelines

## Repository Organization

### Branches
- `main` - Production-ready code
- `develop` - Main development branch
- `feature/*` - Feature branches
- `bugfix/*` - Bug fix branches
- `hotfix/*` - Hot fixes for production
- `release/*` - Release preparation branches

### Branch Protection Rules
1. **main**
   - Require pull request reviews
   - Require status checks to pass
   - No direct pushes
   - No force pushes

2. **develop**
   - Require pull request reviews
   - Require status checks to pass
   - No direct pushes

### Versioning
We follow [Semantic Versioning](https://semver.org/):
- MAJOR.MINOR.PATCH (e.g., 1.0.0)
- Major: Breaking changes
- Minor: New features, backward compatible
- Patch: Bug fixes, backward compatible

### Tags
- Release tags: `v1.0.0`, `v1.1.0`, etc.
- Pre-release tags: `v1.0.0-rc.1`, `v1.0.0-beta.1`

## Development Workflow

### 1. Feature Development
```bash
# Create feature branch
git checkout develop
git checkout -b feature/my-feature

# Work on feature
git add .
git commit -m "feat: description"

# Push and create PR
git push origin feature/my-feature
```

### 2. Bug Fixes
```bash
# Create bugfix branch
git checkout develop
git checkout -b bugfix/issue-description

# Work on fix
git add .
git commit -m "fix: description"

# Push and create PR
git push origin bugfix/issue-description
```

### 3. Hot Fixes
```bash
# Create hotfix branch from main
git checkout main
git checkout -b hotfix/critical-fix

# Work on fix
git add .
git commit -m "fix: description"

# Push and create PR to main AND develop
git push origin hotfix/critical-fix
```

## Commit Messages
We follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation
- `style`: Formatting
- `refactor`: Code restructuring
- `test`: Tests
- `chore`: Maintenance

Examples:
```
feat(api): add user authentication endpoint
fix(db): resolve connection timeout issue
docs: update deployment instructions
```

## Pull Requests

### PR Template
Every PR should include:
1. Description of changes
2. Related issue(s)
3. Type of change
4. Checklist of completed items
5. Testing instructions

### Review Process
1. Code review by at least one team member
2. All status checks must pass
3. All conversations must be resolved
4. Changes requested must be addressed

## Issues

### Issue Labels
- `bug`: Something isn't working
- `enhancement`: New feature or request
- `documentation`: Documentation improvements
- `technical-debt`: Code improvements needed
- `security`: Security concerns
- `priority`: High/Medium/Low
- `good-first-issue`: Good for newcomers

### Issue Template
1. **Bug Report**
   - Description
   - Steps to reproduce
   - Expected behavior
   - Actual behavior
   - Environment details

2. **Feature Request**
   - Problem description
   - Proposed solution
   - Alternative solutions
   - Additional context

## CI/CD Pipeline

### Development Pipeline
1. Code linting
2. Unit tests
3. Integration tests
4. Development deployment

### Production Pipeline
1. Code linting
2. Unit tests
3. Integration tests
4. Security scan
5. Production deployment

## Infrastructure Management

### Development
- Changes through PR only
- Terraform plan in PR comments
- Apply after approval

### Production
- Changes through PR only
- Terraform plan in PR comments
- Manual approval required
- Scheduled maintenance window

## Security

### Secrets Management
- No secrets in repository
- Use GitHub Secrets for CI/CD
- Use AWS Secrets Manager for application

### Access Control
- Limited admin access
- Branch protection rules
- Required reviews
- Required status checks 