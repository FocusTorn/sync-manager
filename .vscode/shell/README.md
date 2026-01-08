# Shell Profile Configuration

This directory contains shell-specific prompt and profile configurations.

## Files

- `prompt.ps1` - PowerShell prompt (zsh-style)
- `prompt.sh` - Git Bash prompt (zsh-style)
- `profile.ps1` - Full PowerShell profile (optional, can source prompt.ps1)
- `load-prompt.sh` - Git Bash loader script

## PowerShell Setup

The PowerShell prompt is automatically loaded by the generic profile loader in your `$PROFILE` when you're in a workspace with a `.vscode` directory.

## Git Bash Setup

To use the Git Bash prompt, add this to your `~/.bashrc` or `~/.bash_profile`:

```bash
# Load workspace-specific prompt if available
if [ -f "$HOME/.vscode/shell/load-prompt.sh" ]; then
    source "$HOME/.vscode/shell/load-prompt.sh"
fi
```

Or, for a more generic approach that works from any workspace:

```bash
# Function to find and load workspace prompt
load_workspace_prompt() {
    local current="$PWD"
    while [ -n "$current" ]; do
        if [ -f "$current/.vscode/shell/prompt.sh" ]; then
            source "$current/.vscode/shell/prompt.sh"
            return 0
        fi
        local parent=$(dirname "$current")
        if [ "$parent" == "$current" ]; then
            break
        fi
        current="$parent"
    done
    return 1
}

# Load prompt when in a workspace
load_workspace_prompt
```

## Features

Both prompts include:
- Time display in `[HH:MM:SS]` format
- Directory display with workspace root as `/` (cyan) or full path (yellow)
- Virtual environment detection (Python venv/conda)
- Detailed git status on the right side:
  - Staged files count (green ✓)
  - Changed files count (yellow ⚠)
  - Deleted files count (red ✘)
  - Untracked files count (gray Ø)
  - Stashed count (blue ⚑)
  - Clean indicator (green ✔)
  - Branch name (blue, bold)
  - Behind/Ahead indicators (↓/↑)

