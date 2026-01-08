#!/bin/bash
# Git Bash version of the zsh-style prompt
# This file should be sourced from .bashrc or .bash_profile

# Get workspace root (go up two levels from .vscode/shell/)
if [[ "${BASH_SOURCE[0]}" == *".vscode/shell/prompt.sh"* ]]; then
    WORKSPACE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
else
    # Try to find workspace root by looking for .vscode directory
    current="$PWD"
    while [ -n "$current" ]; do
        if [ -d "$current/.vscode" ]; then
            WORKSPACE_ROOT="$current"
            break
        fi
        parent=$(dirname "$current")
        if [ "$parent" == "$current" ]; then
            break
        fi
        current="$parent"
    done
    # Fallback
    if [ -z "$WORKSPACE_ROOT" ]; then
        WORKSPACE_ROOT="D:/_dev/_Projects/dev-boards"
    fi
fi

# Color definitions (ANSI escape codes)
# Time color: #33658A
COLOR_TIME='\033[38;2;51;101;138m'
# Directory color (cyan): #86BBD8
COLOR_DIR='\033[38;2;134;187;216m'
# Virtualenv color: #D8A386
COLOR_VENV='\033[38;2;216;163;134m'
# Git prefix/suffix: #D500F9
COLOR_GIT_PREFIX='\033[38;2;213;0;249m'
# Git staged: #06d6a0
COLOR_GIT_STAGED='\033[38;2;6;214;160m'
# Git changed: #ffd166
COLOR_GIT_CHANGED='\033[38;2;255;209;102m'
# Git deleted: #ef476f
COLOR_GIT_DELETED='\033[38;2;239;71;111m'
# Git untracked: #ABABAB
COLOR_GIT_UNTRACKED='\033[38;2;171;171;171m'
# Git branch: #3574AC
COLOR_GIT_BRANCH='\033[38;2;53;116;172m'
# Prompt color: #0277BD
COLOR_PROMPT='\033[38;2;2;119;189m'
# Yellow
COLOR_YELLOW='\033[33m'
# Reset
COLOR_RESET='\033[0m'
COLOR_BOLD='\033[1m'

# Function to get git status
get_git_status() {
    if ! git rev-parse --git-dir > /dev/null 2>&1; then
        return
    fi
    
    local branch=$(git rev-parse --abbrev-ref HEAD 2>/dev/null)
    if [ -z "$branch" ]; then
        return
    fi
    
    # Get status counts
    local staged=0
    local changed=0
    local deleted=0
    local untracked=0
    local stashed=0
    
    # Parse git status
    while IFS= read -r line; do
        if [[ $line =~ ^\?\? ]]; then
            ((untracked++))
        elif [[ $line =~ ^D ]]; then
            ((deleted++))
        elif [[ $line =~ ^[AMRC] ]]; then
            ((staged++))
        elif [[ $line =~ ^.[MD] ]]; then
            ((changed++))
        fi
    done < <(git status --porcelain 2>/dev/null)
    
    # Check for stashes
    if git stash list > /dev/null 2>&1; then
        stashed=$(git stash list | wc -l)
    fi
    
    # Check if clean
    local clean=0
    if [ $staged -eq 0 ] && [ $changed -eq 0 ] && [ $deleted -eq 0 ] && [ $untracked -eq 0 ]; then
        clean=1
    fi
    
    # Get upstream info
    local ahead=0
    local behind=0
    local upstream=$(git rev-parse --abbrev-ref --symbolic-full-name '@{upstream}' 2>/dev/null)
    if [ -n "$upstream" ]; then
        local upstream_info=$(git rev-list --left-right --count "$upstream...HEAD" 2>/dev/null)
        if [ -n "$upstream_info" ]; then
            behind=$(echo "$upstream_info" | cut -f1)
            ahead=$(echo "$upstream_info" | cut -f2)
        fi
    fi
    
    # Build status string
    local status="${COLOR_GIT_PREFIX}"
    
    # File stats FIRST
    if [ $staged -gt 0 ]; then
        status+="${COLOR_GIT_STAGED} ✓${staged}${COLOR_RESET}"
    fi
    if [ $changed -gt 0 ]; then
        status+="${COLOR_GIT_CHANGED} ⚠${changed}${COLOR_RESET}"
    fi
    if [ $deleted -gt 0 ]; then
        status+="${COLOR_GIT_DELETED} ✘${deleted}${COLOR_RESET}"
    fi
    if [ $untracked -gt 0 ]; then
        status+="${COLOR_GIT_UNTRACKED} Ø${untracked}${COLOR_RESET}"
    fi
    if [ $stashed -gt 0 ]; then
        status+="${COLOR_BOLD}\033[34m ⚑${stashed}${COLOR_RESET}"
    fi
    if [ $clean -eq 1 ]; then
        status+="${COLOR_BOLD}\033[32m ✔${COLOR_RESET}"
    fi
    
    # Branch name LAST
    status+="${COLOR_GIT_BRANCH}${COLOR_BOLD}  ${branch}${COLOR_RESET}"
    
    # Upstream info
    if [ $behind -gt 0 ]; then
        status+=" ↓${behind}"
    fi
    if [ $ahead -gt 0 ]; then
        status+=" ↑${ahead}"
    fi
    
    status+="${COLOR_GIT_PREFIX}"
    
    echo -n "$status"
}

# Function to get virtualenv
get_virtualenv() {
    if [ -n "$VIRTUAL_ENV" ]; then
        basename "$VIRTUAL_ENV"
    elif [ -n "$CONDA_DEFAULT_ENV" ]; then
        echo "$CONDA_DEFAULT_ENV"
    fi
}

# Function to format directory
format_dir() {
    local current_path="$PWD"
    local home_path="$HOME"
    local in_workspace=0
    
    # Convert Windows paths to Unix-style for Git Bash if cygpath is available
    if command -v cygpath >/dev/null 2>&1; then
        current_path=$(cygpath -u "$current_path" 2>/dev/null || echo "$current_path")
        workspace_root=$(cygpath -u "$WORKSPACE_ROOT" 2>/dev/null || echo "$WORKSPACE_ROOT")
    else
        workspace_root="$WORKSPACE_ROOT"
    fi
    
    # Check if we're in the workspace
    if [[ "$current_path" == "$workspace_root"* ]]; then
        in_workspace=1
        # Replace workspace path with /
        local relative_path="${current_path#$workspace_root}"
        if [ -z "$relative_path" ]; then
            echo -n "/"
            return
        else
            # Ensure it starts with /
            if [[ ! "$relative_path" =~ ^/ ]]; then
                relative_path="/$relative_path"
            fi
            echo -n "$relative_path"
            return
        fi
    elif [[ "$current_path" == "$home_path"* ]]; then
        # Replace home with ~
        echo -n "${current_path/$home_path/~}"
        return
    else
        echo -n "$current_path"
        return
    fi
}

# Main prompt function
__custom_prompt() {
    # Add blank line above prompt
    echo ""
    
    # Get current time
    local time_str=$(date +"%H:%M:%S")
    
    # Get virtualenv
    local venv=$(get_virtualenv)
    local venv_str=""
    if [ -n "$venv" ]; then
        venv_str="${COLOR_VENV}${venv}${COLOR_RESET} "
    fi
    
    # Get directory
    local display_path=$(format_dir)
    local in_workspace=0
    local current_path="$PWD"
    local workspace_root="$WORKSPACE_ROOT"
    
    # Convert paths for comparison if cygpath is available
    if command -v cygpath >/dev/null 2>&1; then
        current_path=$(cygpath -u "$current_path" 2>/dev/null || echo "$current_path")
        workspace_root=$(cygpath -u "$workspace_root" 2>/dev/null || echo "$workspace_root")
    fi
    
    if [[ "$current_path" == "$workspace_root"* ]]; then
        in_workspace=1
    fi
    
    # Set color based on workspace location
    local dir_color
    if [ $in_workspace -eq 1 ]; then
        dir_color="$COLOR_DIR"
    else
        dir_color="$COLOR_YELLOW"
    fi
    
    # Handle root directory
    local dir_display
    if [ $in_workspace -eq 0 ] && [[ "$display_path" == "/" ]]; then
        dir_display="${COLOR_YELLOW}[ROOT: /]${COLOR_RESET}"
    else
        dir_display="${dir_color}[${display_path}]${COLOR_RESET}"
    fi
    
    # Get git status
    local git_status=$(get_git_status)
    
    # Build left prompt
    local left_prompt="${COLOR_TIME}[${time_str}]${COLOR_RESET} ${venv_str}${dir_display}"
    
    # Calculate right prompt position (RPROMPT equivalent)
    # In bash, we need to use PS1 with escape sequences for right alignment
    # For now, we'll put git status on the same line after the directory
    
    # Set PS1 with right-aligned git status
    if [ -n "$git_status" ]; then
        # Use printf to calculate spacing for right alignment
        local term_width=$(tput cols 2>/dev/null || echo 80)
        local left_len=$(echo -n "$left_prompt" | sed 's/\x1b\[[0-9;]*m//g' | wc -c)
        local git_len=$(echo -n "$git_status" | sed 's/\x1b\[[0-9;]*m//g' | wc -c)
        local spaces=$((term_width - left_len - git_len - 1))
        
        if [ $spaces -gt 0 ]; then
            PS1="\n${left_prompt}$(printf '%*s' $spaces)${git_status}\n${COLOR_PROMPT}\$${COLOR_RESET} "
        else
            PS1="\n${left_prompt}\n${git_status}\n${COLOR_PROMPT}\$${COLOR_RESET} "
        fi
    else
        PS1="\n${left_prompt}\n${COLOR_PROMPT}\$${COLOR_RESET} "
    fi
}

# Set the prompt
PROMPT_COMMAND='__custom_prompt'

