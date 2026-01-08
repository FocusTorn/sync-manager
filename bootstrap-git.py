#!/usr/bin/env python3
"""
Cross-platform Git bootstrap script for workspace setup.

This script sets up Git for a workspace by:
1. Checking if Git is installed
2. Checking if workspace is already a Git repository
3. Initializing Git repository if needed
4. Setting up basic Git configuration (user.name, user.email)
5. Creating .gitignore file if it doesn't exist
6. Making initial commit if there are changes

Usage:
    python bootstrap-git.py [--skip-config] [--skip-gitignore] [--skip-commit]
"""

import subprocess
import sys
import platform
from pathlib import Path
from typing import List, Optional, Tuple

# Fix Windows console encoding
if sys.platform == "win32":
    import io
    reconfigure_stdout = getattr(sys.stdout, 'reconfigure', None)
    reconfigure_stderr = getattr(sys.stderr, 'reconfigure', None)
    if reconfigure_stdout is not None:
        reconfigure_stdout(encoding='utf-8')
    if reconfigure_stderr is not None:
        reconfigure_stderr(encoding='utf-8')
    if reconfigure_stdout is None or reconfigure_stderr is None:
        sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8', errors='replace')
        sys.stderr = io.TextIOWrapper(sys.stderr.buffer, encoding='utf-8', errors='replace')

# ANSI color codes for terminal output (matching outerm specifications)
RESET = "\x1B[0m"
ERROR_COLOR = "\x1B[38;5;196m"      # Red
WARNING_COLOR = "\x1B[38;5;220m"    # Yellow
INFO_COLOR = "\x1B[38;5;39m"        # Blue
SUCCESS_COLOR = "\x1B[38;5;46m"     # Green
ACTION_COLOR = "\x1B[38;5;66m"      # Teal
HEADER_COLOR = "\x1B[38;5;27m"      # Beige/Warm gray
HEADER_BOLD = "\x1B[1m"
TITLE_COLOR = "\x1B[38;5;144m"      # Cyan

# Status message functions using ANSI codes
def error(colored_text: str, non_colored_text: str = '') -> str:
    """Format error message with red ✗ icon."""
    return f"{ERROR_COLOR}✗{RESET} {colored_text}{non_colored_text}"


def warning(colored_text: str, non_colored_text: str = '') -> str:
    """Format warning message with yellow ⚠ icon (entire line colored)."""
    return f"{WARNING_COLOR}⚠ {colored_text}{non_colored_text}{RESET}"


def info(colored_text: str, non_colored_text: str = '') -> str:
    """Format info message with blue ｉ icon (bold, no space after icon)."""
    return f"{HEADER_BOLD}{INFO_COLOR}ｉ{RESET}{INFO_COLOR}{colored_text}{non_colored_text}{RESET}"


def success(colored_text: str, non_colored_text: str = '') -> str:
    """Format success message with green ✔ icon."""
    return f"{SUCCESS_COLOR}✔{RESET} {colored_text}{non_colored_text}"


def action(colored_text: str, non_colored_text: str = '') -> str:
    """Format action message with teal ⮻ icon."""
    return f"{ACTION_COLOR}⮻{RESET} {colored_text}{non_colored_text}"


class write_header:
    """Context manager for standard header (┌─)."""
    def __init__(self, title: str):
        self.title = title
    
    def __enter__(self):
        dashes = 75 - len(self.title) - 5  # 5 chars for "┌─  ─"
        print(f"{HEADER_COLOR}┌─ {self.title} ─" + "─" * dashes + RESET)
        return self
    
    def __exit__(self, *args):
        print(f"{HEADER_COLOR}└─" + "─" * 75 + RESET)


class write_header_fat:
    """Context manager for fat header (┏━)."""
    def __init__(self, title: str):
        self.title = title
    
    def __enter__(self):
        dashes = 75 - len(self.title) - 5  # 5 chars for "┏━  ━"
        print(f"{HEADER_BOLD}{HEADER_COLOR}┏━ {self.title} ━" + "━" * dashes + RESET)
        return self
    
    def __exit__(self, *args):
        print(f"{HEADER_BOLD}{HEADER_COLOR}┗━" + "━" * 75 + RESET)


def write_boxed_header(title: str, width: int = 80):
    """Write boxed header for titles."""
    box_line = "━" * (width - 2)
    print(f"{TITLE_COLOR}┏{box_line}┓{RESET}")
    padding = (width - len(title) - 2) // 2
    title_line = " " * padding + title + " " * (width - len(title) - padding - 2)
    print(f"{TITLE_COLOR}┃{title_line}┃{RESET}")
    print(f"{TITLE_COLOR}┗{box_line}┛{RESET}")


def is_windows() -> bool:
    """Check if running on Windows."""
    return platform.system() == "Windows"


def check_command_exists(cmd: str) -> bool:
    """Check if a command exists in PATH."""
    try:
        if is_windows():
            result = subprocess.run(
                ["where", cmd] if cmd != "git" else ["git", "version"],
                capture_output=True,
                timeout=5,
                check=False
            )
            return result.returncode == 0
        else:
            result = subprocess.run(
                ["which", cmd],
                capture_output=True,
                timeout=5,
                check=False
            )
            return result.returncode == 0
    except (FileNotFoundError, subprocess.TimeoutExpired):
        return False


def run_git_command(cmd: List[str], cwd: Path, check: bool = True) -> Tuple[int, str, str]:
    """Run a git command and return exit code, stdout, stderr."""
    try:
        result = subprocess.run(
            ["git"] + cmd,
            cwd=cwd,
            capture_output=True,
            timeout=30,
            text=True,
            check=check
        )
        return result.returncode, result.stdout.strip(), result.stderr.strip()
    except subprocess.TimeoutExpired:
        return 1, "", "Command timed out"
    except FileNotFoundError:
        return 1, "", "Git is not installed or not in PATH"


def check_git_installed() -> bool:
    """Check if Git is installed."""
    return check_command_exists("git")


def is_git_repo(path: Path) -> bool:
    """Check if a path is a Git repository.
    
    Uses git rev-parse --git-dir for reliable detection, with fallback
    to checking for .git directory/file.
    """
    path = Path(path).resolve()
    
    if not path.exists() or not path.is_dir():
        return False
    
    # First try: Use git command for reliable detection (handles worktrees, submodules, etc.)
    try:
        exit_code, stdout, _ = run_git_command(['rev-parse', '--git-dir'], path, check=False)
        if exit_code == 0 and stdout and stdout.strip():
            return True
    except Exception:
        pass
    
    # Fallback: Check for .git directory or file
    git_path = path / '.git'
    if git_path.exists():
        return True
    
    return False


def get_git_config(key: str) -> Optional[str]:
    """Get a Git configuration value."""
    exit_code, stdout, _ = run_git_command(['config', '--global', key], Path.cwd(), check=False)
    if exit_code == 0 and stdout:
        return stdout.strip()
    return None


def set_git_config(key: str, value: str, global_config: bool = True) -> bool:
    """Set a Git configuration value."""
    cmd = ['config']
    if global_config:
        cmd.append('--global')
    cmd.extend([key, value])
    
    exit_code, _, stderr = run_git_command(cmd, Path.cwd(), check=False)
    if exit_code == 0:
        return True
    else:
        print(error(f"Failed to set {key}: {stderr}"))
        return False


def setup_git_config() -> bool:
    """Set up basic Git configuration (user.name, user.email) if not already set."""
    with write_header("Git Configuration"):
        # Check user.name
        user_name = get_git_config('user.name')
        if not user_name:
            print(info("User name is not configured"))
            user_name = input("Enter your Git user name: ").strip()
            if not user_name:
                print(error("User name is required"))
                return False
            if not set_git_config('user.name', user_name):
                return False
            print(success(f"Set user.name to: {user_name}"))
        else:
            print(success(f"User name already configured: {user_name}"))
        
        # Check user.email
        user_email = get_git_config('user.email')
        if not user_email:
            print()
            print(info("User email is not configured"))
            user_email = input("Enter your Git user email: ").strip()
            if not user_email:
                print(error("User email is required"))
                return False
            if not set_git_config('user.email', user_email):
                return False
            print(success(f"Set user.email to: {user_email}"))
        else:
            print(success(f"User email already configured: {user_email}"))
        
        return True


def create_default_gitignore(workspace_path: Path) -> bool:
    """Create a default .gitignore file if it doesn't exist."""
    gitignore_path = workspace_path / '.gitignore'
    
    if gitignore_path.exists():
        print(success(".gitignore already exists"))
        return True
    
    # Default .gitignore content for common development environments
    default_gitignore = """# Python
__pycache__/
*.py[cod]
*$py.class
*.so
.Python
build/
develop-eggs/
dist/
downloads/
eggs/
.eggs/
lib/
lib64/
parts/
sdist/
var/
wheels/
*.egg-info/
.installed.cfg
*.egg
MANIFEST
.venv/
venv/
ENV/
env/

# Rust
target/
**/*.rs.bk
Cargo.lock

# Node.js
node_modules/
npm-debug.log*
yarn-debug.log*
yarn-error.log*
.pnpm-debug.log*

# IDEs and editors
.vscode/
.cursor/
.idea/
*.swp
*.swo
*~
.DS_Store
*.sublime-project
*.sublime-workspace

# OS
Thumbs.db
.DS_Store
*.log

# Build artifacts
*.o
*.exe
*.dll
*.dylib
*.class

# Testing
.pytest_cache/
.coverage
htmlcov/
.tox/

# Misc
*.bak
*.tmp
*.temp
"""
    
    try:
        gitignore_path.write_text(default_gitignore, encoding='utf-8')
        print(success(f"Created .gitignore"))
        return True
    except Exception as e:
        print(error(f"Failed to create .gitignore: {e}"))
        return False


def init_git_repo(workspace_path: Path) -> bool:
    """Initialize a Git repository."""
    try:
        exit_code, _, stderr = run_git_command(['init'], workspace_path, check=False)
        if exit_code == 0:
            print(success("Git repository initialized"))
            return True
        else:
            print(error(f"Failed to initialize repository: {stderr}"))
            return False
    except Exception as e:
        print(error(f"Error initializing repository: {e}"))
        return False


def has_uncommitted_changes(workspace_path: Path) -> bool:
    """Check if there are uncommitted changes."""
    exit_code, stdout, _ = run_git_command(['status', '--porcelain'], workspace_path, check=False)
    if exit_code == 0:
        return len(stdout.strip()) > 0
    return False


def make_initial_commit(workspace_path: Path) -> bool:
    """Make an initial commit if there are changes."""
    if not has_uncommitted_changes(workspace_path):
        print(info("No changes to commit"))
        return True
    
    # Stage all files
    exit_code, _, stderr = run_git_command(['add', '.'], workspace_path, check=False)
    if exit_code != 0:
        print(error(f"Failed to stage files: {stderr}"))
        return False
    
    # Check if there's already a commit
    exit_code, _, _ = run_git_command(['rev-parse', '--verify', 'HEAD'], workspace_path, check=False)
    if exit_code == 0:
        print(info("Repository already has commits"))
        return True
    
    # Make initial commit
    exit_code, _, stderr = run_git_command(
        ['commit', '-m', 'Initial commit'],
        workspace_path,
        check=False
    )
    if exit_code == 0:
        print(success("Initial commit created"))
        return True
    else:
        print(error(f"Failed to create initial commit: {stderr}"))
        return False


def main():
    """Main function."""
    import argparse
    
    parser = argparse.ArgumentParser(
        description='Bootstrap Git for a workspace',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  python bootstrap-git.py
  python bootstrap-git.py --skip-config
  python bootstrap-git.py --skip-gitignore --skip-commit
        """
    )
    parser.add_argument(
        '--skip-config',
        action='store_true',
        help='Skip Git configuration (user.name, user.email)'
    )
    parser.add_argument(
        '--skip-gitignore',
        action='store_true',
        help='Skip creating .gitignore file'
    )
    parser.add_argument(
        '--skip-commit',
        action='store_true',
        help='Skip making initial commit'
    )
    parser.add_argument(
        '--workspace-path',
        type=str,
        help='Path to workspace (default: current directory)'
    )
    
    args = parser.parse_args()
    
    # Determine workspace path
    workspace_path = Path(args.workspace_path).resolve() if args.workspace_path else Path.cwd()
    
    # Display header
    write_boxed_header("Git Workspace Bootstrap", width=80)
    print()
    
    # Check prerequisites
    with write_header_fat("Prerequisites"):
        if not check_git_installed():
            print(error("Git is not installed"))
            print()
            print("Please install Git:")
            if is_windows():
                print("  - Windows: Download from https://git-scm.com/download/win")
                print("  - Or use winget: winget install --id Git.Git -e --source winget")
            else:
                print("  - macOS: brew install git")
                print("  - Linux: sudo apt-get install git (Ubuntu/Debian)")
            return 1
        
        print(success("Git is installed"))
        print(info(f"Workspace path: {workspace_path}"))
        print()
    
    # Check if already a git repository
    with write_header_fat("Repository Status"):
        if is_git_repo(workspace_path):
            print(success(f"Workspace is already a Git repository"))
            print(info("Skipping initialization"))
        else:
            print(action("Initializing Git repository..."))
            if not init_git_repo(workspace_path):
                return 1
        print()
    
    # Set up Git configuration
    if not args.skip_config:
        with write_header_fat("Git Configuration"):
            if not setup_git_config():
                return 1
            print()
    
    # Create .gitignore
    if not args.skip_gitignore:
        with write_header_fat(".gitignore Setup"):
            if not create_default_gitignore(workspace_path):
                return 1
            print()
    
    # Make initial commit
    if not args.skip_commit:
        with write_header_fat("Initial Commit"):
            if not make_initial_commit(workspace_path):
                return 1
            print()
    
    # Summary
    with write_header_fat("Summary"):
        print(success("Git workspace setup complete!"))
        print()
        print("Next steps:")
        print("  1. Review and customize .gitignore if needed")
        print("  2. Add a remote repository:")
        print("     git remote add origin <repository-url>")
        print("  3. Push to remote:")
        print("     git push -u origin main")
        print()
    
    return 0


if __name__ == '__main__':
    try:
        sys.exit(main())
    except KeyboardInterrupt:
        print()
        print(error("Interrupted by user"))
        sys.exit(1)
    except Exception as e:
        print(error(f"Unexpected error: {e}"))
        sys.exit(1)
