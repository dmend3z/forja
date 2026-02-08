# forja-cli

Skills marketplace CLI for [Claude Code](https://claude.com/claude-code). Install curated skills, agents, and team configs organized around a 5-phase development workflow.

## Install

```bash
npm install -g forja-cli
```

## Usage

```bash
# Initialize (clones the skills registry)
forja init

# Browse available skills
forja phases
forja list --available

# Install a skill
forja install test/tdd/workflow

# Check installation health
forja doctor
```

## Supported Platforms

| Platform | Architecture |
|----------|-------------|
| macOS | ARM64 (Apple Silicon) |
| macOS | x64 (Intel) |
| Linux | x64 |
| Linux | ARM64 |

## More Info

See the full documentation at [github.com/dmend3z/forja](https://github.com/dmend3z/forja).
