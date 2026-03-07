# Devbox

A practical guide to Devbox for the cl-institute project.

## What is Devbox?

Devbox is a CLI tool by Jetify that creates reproducible, isolated development environments using Nix packages under the hood — without requiring you to learn Nix itself. Think of it as a way to say "here's exactly what my project needs" and have everyone (including CI/CD systems and AI agents) get the identical setup instantly.

## Why We're Using It

The cl-institute project is building an "Intelligent Institution" system in TypeScript with autonomous agents that need a consistent runtime environment. We need:

- Node.js 24
- SQLite
- Git
- Various build tools and utilities

Without Devbox, every developer would need to manually install and maintain these dependencies, leading to "works on my machine" problems. With Devbox, we declare everything once in `devbox.json`, and anyone who runs `devbox shell` gets the identical environment — same versions, same tools, same behavior.

This is especially important when working with AI coding agents, which need predictable environments to work effectively.

## Key Concepts

### `devbox.json`
The manifest file that lists everything your environment needs:
- Packages (e.g., `nodejs@24`, `sqlite`)
- Environment variables
- Init hooks (commands that run when entering the shell)
- Named scripts (shortcuts for common commands)

### `devbox.lock`
Auto-generated lockfile that pins exact package versions and their dependencies. **Commit this to git** — it ensures everyone gets identical package versions.

### `devbox shell`
Drops you into a shell with all declared packages available. Your system's global packages are hidden; only Devbox packages are in scope. Exit with `exit` or Ctrl+D.

### `devbox run <script>`
Runs a named script from `devbox.json`. Scripts can be simple commands or multi-line shell scripts.

### Managing Packages
```bash
devbox add nodejs@24     # Add a package
devbox add nodejs        # Add latest version
devbox rm sqlite         # Remove a package
devbox search postgresql # Find available packages
```

### Docker Generation
```bash
devbox generate dockerfile
```
Generates a Dockerfile from your Devbox environment. This bridges local development and containerization — define your environment once, use it everywhere.

## How It Relates to Docker

**Devbox** defines *what* packages and tools you need.  
**Docker** defines *how* to run containers (isolation, networking, volume mounts, orchestration).

They complement each other:
- Use Devbox for local development and CI/CD environments
- Use Docker for deployment, service orchestration, and production
- Use `devbox generate dockerfile` to create Docker images with the same packages

Many projects use Devbox to replace Docker Compose for local development because it's faster and doesn't require container overhead for simple tasks.

## How It Works Under the Hood

Devbox uses the **Nix package manager** to:
1. Download packages from nixpkgs (a repository with 100,000+ packages)
2. Install them into an isolated Nix store (`/nix/store/`)
3. Create a shell environment where only those packages are visible

**You don't need to learn Nix.** Devbox abstracts away the complexity:
- No `.nix` files to write
- No Nix expression language to learn
- Just JSON configuration and simple CLI commands

Packages don't pollute your system PATH outside the Devbox shell. When you `exit`, your normal environment is restored.

## Installation

```bash
curl -fsSL https://get.jetify.com/devbox | bash
```

The installer:
- Downloads the Devbox binary
- Installs Nix if not already present
- Adds Devbox to your PATH

Restart your shell after installation.

## Common Workflow

### Starting a New Project
```bash
devbox init                # Creates devbox.json
devbox add nodejs@24       # Add Node.js 24
devbox add sqlite          # Add SQLite
devbox shell               # Enter the environment
node --version             # Verify packages work
exit                       # Leave the environment
```

### Working on an Existing Project
```bash
git clone <repository>
cd <project>
devbox shell               # Installs packages from devbox.json automatically
npm install                # Run project-specific setup
devbox run test            # Run project scripts
```

### Daily Development
```bash
devbox shell               # Enter the environment
# ... do your work ...
devbox run lint            # Run scripts without leaving shell
devbox run test
exit                       # Done for the day
```

### Adding New Dependencies
```bash
devbox search postgres     # Find the right package name
devbox add postgresql      # Add it
devbox shell               # Restart shell to use new package
```

## Tips & Tricks

### Run Commands Without Entering Shell
```bash
devbox run build           # Runs a script
devbox run -- npm test     # Run arbitrary commands in the environment
```

### Check What's Installed
```bash
devbox list                # Show all packages
devbox info nodejs         # Show details about a package
```

### Update Packages
```bash
devbox update              # Updates all packages to latest compatible versions
```

### Troubleshooting
If packages aren't available or commands fail:
```bash
exit                       # Leave the shell
devbox shell               # Re-enter to refresh
```

If you see Nix errors:
```bash
devbox cache clear         # Clear the package cache
devbox shell               # Try again
```

## Learn More

- [Devbox Documentation](https://www.jetify.com/devbox/docs/)
- [Browse Available Packages](https://www.nixhub.io/)
- [Devbox Examples](https://github.com/jetify-com/devbox-examples)

## For cl-institute Developers

Our `devbox.json` is the single source of truth for development dependencies. When you:
- Add a new tool or runtime version, update `devbox.json`
- See a package version change in a PR, review `devbox.lock` changes
- Onboard a new developer, they run `devbox shell` — that's it

AI agents in this project use Devbox to ensure they have the right tools available. When debugging agent behavior, always check you're in a Devbox shell — otherwise they might be using different package versions than you are.
