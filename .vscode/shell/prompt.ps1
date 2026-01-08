# ANSI Color codes matching your zsh theme
$ESC = [char]27
function Color { param($code) "$ESC[$code" }

# Get workspace root
# If prompt.ps1 is in .vscode\shell\prompt.ps1, go up two levels to get workspace root
# Otherwise, assume it's in the workspace root
if ($PSScriptRoot) {
    if ($PSScriptRoot -match '\.vscode\\shell$') {
        # In .vscode\shell, go up two levels
        $script:WorkspaceRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
    } else {
        # Assume it's in the workspace root
        $script:WorkspaceRoot = $PSScriptRoot
    }
} else {
    # Fallback
    $script:WorkspaceRoot = "D:\_dev\_Projects\dev-boards"
}




# OMZ Style Prompt ---------------------------------------->> 

# Color definitions
$ColorTime = "38;2;51;101;138"      # #33658A
$ColorDir = "38;2;134;187;216"      # #86BBD8
$ColorVenv = "38;2;216;163;134"     # #D8A386
$ColorGitPrefix = "38;2;213;0;249"  # #D500F9
$ColorGitStaged = "38;2;6;214;160"  # #06d6a0
$ColorGitChanged = "38;2;255;209;102" # #ffd166
$ColorGitDeleted = "38;2;239;71;111" # #ef476f
$ColorGitUntracked = "38;2;171;171;171" # #ABABAB
$ColorGitBranch = "38;2;53;116;172" # #3574AC
$ColorPrompt = "38;2;2;119;189"     # #0277BD

function Get-GitStatus {
    if (-not (Test-Path .git)) { return $null }
    
    try {
        # Get git status porcelain output
        $statusOutput = git status --porcelain 2>$null
        if ($LASTEXITCODE -ne 0) { return $null }
        
        # Get branch name
        $branch = git rev-parse --abbrev-ref HEAD 2>$null
        if ($LASTEXITCODE -ne 0) { return $null }
        
        # Count file statuses
        $staged = 0
        $changed = 0
        $deleted = 0
        $untracked = 0
        
        if ($statusOutput) {
            $lines = $statusOutput -split "`n"
            foreach ($line in $lines) {
                if ($line.Length -lt 2) { continue }
                
                $indexStatus = $line[0]
                $workingStatus = $line[1]
                
                if ($line -match '^\?\?') {
                    # Untracked files
                    $untracked++
                } elseif ($indexStatus -eq 'D' -or $workingStatus -eq 'D') {
                    # Deleted files (staged or unstaged)
                    $deleted++
                } elseif ($indexStatus -in @('A', 'M', 'R', 'C')) {
                    # Staged files (Added, Modified, Renamed, Copied)
                    $staged++
                } elseif ($workingStatus -eq 'M') {
                    # Modified in working tree (not staged)
                    $changed++
                }
            }
        }
        
        # Check for stashed changes
        $stashed = 0
        $stashOutput = git stash list 2>$null
        if ($LASTEXITCODE -eq 0 -and $stashOutput) {
            $stashed = ($stashOutput -split "`n").Count
        }
        
        # Check if clean
        $clean = ($staged -eq 0 -and $changed -eq 0 -and $deleted -eq 0 -and $untracked -eq 0)
        
        # Get upstream info
        $ahead = 0
        $behind = 0
        # Get upstream branch name first
        $upstreamBranch = git rev-parse --abbrev-ref --symbolic-full-name '@{upstream}' 2>$null
        if ($LASTEXITCODE -eq 0 -and $upstreamBranch) {
            $upstream = git rev-list --left-right --count "$upstreamBranch...HEAD" 2>$null
            if ($LASTEXITCODE -eq 0 -and $upstream) {
                $parts = $upstream -split "`t"
                if ($parts.Count -eq 2) {
                    $behind = [int]$parts[0]
                    $ahead = [int]$parts[1]
                }
            }
        }
        
        return @{
            Branch = $branch
            Staged = $staged
            Changed = $changed
            Deleted = $deleted
            Untracked = $untracked
            Stashed = $stashed
            Clean = $clean
            Ahead = $ahead
            Behind = $behind
        }
    } catch {
        return $null
    }
}

function Get-VirtualEnv {
    if ($env:VIRTUAL_ENV) {
        $venvName = Split-Path -Leaf $env:VIRTUAL_ENV
        return $venvName
    }
    if ($env:CONDA_DEFAULT_ENV) {
        return $env:CONDA_DEFAULT_ENV
    }
    return $null
}

function Format-GitStatus {
    param($gitStatus)
    
    if (-not $gitStatus) { return "" }
    
    # Unicode characters for git status (matching zsh theme)
    # Using Nerd Font icons (Private Use Area: U+E000-U+F8FF)
    # If Nerd Font is not available, these may not render correctly
    $stagedChar = "Ξ" # 🞠 ⏏ Ξ  [char]0xE727      #  (git staged - Nerd Font U+E727)
    $changedChar = "" # [char]0xE728     #  (git modified - Nerd Font U+E728)
    $deletedChar = "✘"              # ✘ (U+2718)
    $untrackedChar = "Ø"            # Ø (U+00D8)
    $stashedChar = "󰴮" # [char]0xF4B6     #  (stash icon - Nerd Font U+F4B6, fallback: ⚑ U+2691)
    $branchChar = "" # [char]0xE0A0      #  (branch icon - Nerd Font U+E0A0)
    
    # Half-width space (en space - U+2002, half an em width)
    $halfSpace = [char]0x200A
    
    $status = ""
    
    
    # Prefix
    $status += "$ESC[$($ColorGitPrefix)m"
    
    # File stats FIRST
    if ($gitStatus.Staged -gt 0) {
        
        # $status += "$ESC[$($ColorGitStaged)m $ESC[1m$stagedChar$ESC[22m$($gitStatus.Staged)$ESC[0m"
        $status += "$ESC[$($ColorGitStaged)m $stagedChar$($gitStatus.Staged)$ESC[0m"
        
    }
    if ($gitStatus.Changed -gt 0) {
        $status += "$ESC[$($ColorGitChanged)m $changedChar$($gitStatus.Changed)$ESC[0m"
    }
    if ($gitStatus.Deleted -gt 0) {
        $status += "$ESC[$($ColorGitDeleted)m $deletedChar$($gitStatus.Deleted)$ESC[0m"
    }
    if ($gitStatus.Untracked -gt 0) {
        $status += "$ESC[$($ColorGitUntracked)m $untrackedChar$($gitStatus.Untracked)$ESC[0m"
    }
    if ($gitStatus.Stashed -gt 0) {
        $status += "$ESC[1;34m $stashedChar$($gitStatus.Stashed)$ESC[0m"
    }
    if ($gitStatus.Clean) {
        $status += "$ESC[1;32m ✔$ESC[0m"
    }
    
    # Branch name LAST with branch icon
    $status += "$ESC[$($ColorGitBranch)m $branchChar$($gitStatus.Branch)$ESC[0m"
    
    # Upstream info
    if ($gitStatus.Behind -gt 0) {
        $status += " ↓$($gitStatus.Behind)"
    }
    if ($gitStatus.Ahead -gt 0) {
        $status += " ↑$($gitStatus.Ahead)"
    }
    
    # Suffix
    $status += "$ESC[$($ColorGitPrefix)m"
    
    return $status
}

function prompt {
    # Add blank line above prompt
    Write-Host ""
    
    # Get current time in [HH:MM:SS] format
    $time = Get-Date -Format "HH:mm:ss"
    
    # Get virtualenv
    $venv = Get-VirtualEnv
    $venvStr = if ($venv) { "$ESC[$($ColorVenv)m$venv$ESC[0m " } else { "" }
    
    # Get current directory
    $currentPath = $PWD.Path
    $homePath = $HOME
    $inWorkspace = $false
    
    # Check if we're in the dev-boards workspace
    if ($script:WorkspaceRoot -and $currentPath.StartsWith($script:WorkspaceRoot)) {
        $inWorkspace = $true
        # Replace workspace path with /
        $relativePath = $currentPath.Substring($script:WorkspaceRoot.Length)
        if ([string]::IsNullOrWhiteSpace($relativePath)) {
            $displayPath = "/"
        } else {
            # Normalize path separators and ensure it starts with /
            $relativePath = $relativePath.Replace('\', '/')
            if (-not $relativePath.StartsWith('/')) {
                $relativePath = "/" + $relativePath
            }
            $displayPath = $relativePath
        }
    } elseif ($currentPath.StartsWith($homePath)) {
        # Replace home with ~
        $displayPath = $currentPath.Replace($homePath, "~")
        # Normalize path separators for display (use / like Unix)
        $displayPath = $displayPath.Replace('\', '/')
    } else {
        $displayPath = $currentPath
        # Normalize path separators for display (use / like Unix)
        $displayPath = $displayPath.Replace('\', '/')
    }
    
    # Set color based on workspace location: cyan if in workspace, yellow if outside
    if ($inWorkspace) {
        $dirColor = $ColorDir  # Cyan (#86BBD8)
    } else {
        $dirColor = "33"  # Yellow
    }
    
    # Handle root directory (only if NOT in workspace)
    if (-not $inWorkspace -and ($displayPath -eq "\" -or $displayPath -eq "/")) {
        $dirDisplay = "$ESC[33m[ROOT: /]$ESC[0m"
    } else {
        $dirDisplay = "$ESC[$($dirColor)m[$displayPath]$ESC[0m"
    }
    
    # Get git status
    $gitStatus = Get-GitStatus
    $gitStatusStr = Format-GitStatus $gitStatus
    
    # Build the prompt lines
    $timeStr = "$ESC[$($ColorTime)m[$time]$ESC[0m"
    # Use a simpler prompt character if the special one doesn't render
    $promptChar = "$ESC[$($ColorPrompt)m❯$ESC[0m"
    
    # Calculate right prompt position for git status
    $leftPrompt = "$timeStr $venvStr$dirDisplay"
    $rightPrompt = $gitStatusStr
    
    # Write left prompt
    Write-Host $leftPrompt -NoNewline
    
    # Write right prompt (RPROMPT equivalent)
    if ($rightPrompt) {
        try {
            $cursorPos = $Host.UI.RawUI.CursorPosition
            $windowWidth = $Host.UI.RawUI.WindowSize.Width
            # Remove ANSI codes to get actual text length
            $rightPromptPlain = $rightPrompt -replace "$ESC\[[^m]*m", ""
            $rightPromptLength = $rightPromptPlain.Length
            $newX = [Math]::Max(0, $windowWidth - $rightPromptLength - 1)
            
            if ($newX -gt $cursorPos.X) {
                $Host.UI.RawUI.CursorPosition = New-Object System.Management.Automation.Host.Coordinates($newX, $cursorPos.Y)
                Write-Host $rightPrompt -NoNewline
            } else {
                Write-Host ""
                Write-Host $rightPrompt -NoNewline
            }
        } catch {
            # Fallback if cursor positioning fails
            Write-Host ""
            Write-Host $rightPrompt -NoNewline
        }
        # Always write a newline after right prompt to ensure prompt character is on next line
        Write-Host ""
    } else {
        Write-Host ""
    }
    
    # Return the prompt character (will appear on a new line)
    return "$promptChar "
}

#-----------------------------------------------------------------------------<<

# PowerShell Profile Paths Alias
function Show-PowerShellProfiles { #>
    
    # .SYNOPSIS
    # Displays all PowerShell profile paths for PowerShell 5 and PowerShell 7.
    # Paths are formatted to be clickable in VS Code integrated terminal.
    
    
    Write-Host "`nPowerShell Profile Paths" -ForegroundColor Cyan
    Write-Host "========================`n" -ForegroundColor Cyan
    
    $currentPSVersion = $PSVersionTable.PSVersion.Major
    
    # Helper function to convert paths with spaces to junction paths without spaces
    function Convert-PathToJunction {
        param([string]$Path)
        if (-not $Path) { return $Path }
        
        # Check if path contains "z Linked from C" and convert to "zLinkedfromC"
        if ($Path -match '\\z Linked from C\\') {
            $junctionPath = $Path -replace '\\z Linked from C\\', '\zLinkedfromC\'
            # Verify the junction exists
            if (Test-Path $junctionPath) {
                return $junctionPath
            }
        }
        return $Path
    }
    
    # Helper function to format path as clickable link for VS Code
    function Format-ClickablePath {
        param([string]$Path, [string]$Label)
        if ($Path) {
            # Convert to junction path (without spaces) if available
            $displayPath = Convert-PathToJunction -Path $Path
            
            # Output label and path on one line: label in white, path in cyan
            Write-Host "  $Label " -ForegroundColor White -NoNewline
            Write-Host $displayPath -ForegroundColor Cyan
        } else {
            Write-Host "  $Label " -ForegroundColor White -NoNewline
            Write-Host "(not set)" -ForegroundColor DarkGray
        }
    }
    
    # PowerShell 5 (Windows PowerShell)
    if ($currentPSVersion -eq 5) {
        Write-Host "PowerShell 5 (Windows PowerShell) - Current:" -ForegroundColor Yellow
        Format-ClickablePath -Path $PROFILE.CurrentUserCurrentHost -Label "Current User, Current Host:"
        Format-ClickablePath -Path $PROFILE.CurrentUserAllHosts -Label "Current User, All Hosts:"
        Format-ClickablePath -Path $PROFILE.AllUsersCurrentHost -Label "All Users, Current Host:"
        Format-ClickablePath -Path $PROFILE.AllUsersAllHosts -Label "All Users, All Hosts:"
    } else {
        Write-Host "PowerShell 5 (Windows PowerShell):" -ForegroundColor Yellow
        if (Get-Command powershell.exe -ErrorAction SilentlyContinue) {
            $ps5Profiles = powershell.exe -NoProfile -Command {
                @{
                    CurrentUserCurrentHost = $PROFILE.CurrentUserCurrentHost
                    CurrentUserAllHosts = $PROFILE.CurrentUserAllHosts
                    AllUsersCurrentHost = $PROFILE.AllUsersCurrentHost
                    AllUsersAllHosts = $PROFILE.AllUsersAllHosts
                } | ConvertTo-Json
            } | ConvertFrom-Json
            
            Format-ClickablePath -Path $ps5Profiles.CurrentUserCurrentHost -Label "Current User, Current Host:"
            Format-ClickablePath -Path $ps5Profiles.CurrentUserAllHosts -Label "Current User, All Hosts:"
            Format-ClickablePath -Path $ps5Profiles.AllUsersCurrentHost -Label "All Users, Current Host:"
            Format-ClickablePath -Path $ps5Profiles.AllUsersAllHosts -Label "All Users, All Hosts:"
        } else {
            Write-Host "  Not available" -ForegroundColor Gray
        }
    }
    
    Write-Host ""
    
    # PowerShell 7 (pwsh)
    if ($currentPSVersion -ge 7) {
        Write-Host "PowerShell 7 (pwsh) - Current:" -ForegroundColor Yellow
        Format-ClickablePath -Path $PROFILE.CurrentUserCurrentHost -Label "Current User, Current Host:"
        Format-ClickablePath -Path $PROFILE.CurrentUserAllHosts -Label "Current User, All Hosts:"
        Format-ClickablePath -Path $PROFILE.AllUsersCurrentHost -Label "All Users, Current Host:"
        Format-ClickablePath -Path $PROFILE.AllUsersAllHosts -Label "All Users, All Hosts:"
    } else {
        Write-Host "PowerShell 7 (pwsh):" -ForegroundColor Yellow
        if (Get-Command pwsh -ErrorAction SilentlyContinue) {
            $pwshProfiles = pwsh -NoProfile -Command {
                @{
                    CurrentUserCurrentHost = $PROFILE.CurrentUserCurrentHost
                    CurrentUserAllHosts = $PROFILE.CurrentUserAllHosts
                    AllUsersCurrentHost = $PROFILE.AllUsersCurrentHost
                    AllUsersAllHosts = $PROFILE.AllUsersAllHosts
                } | ConvertTo-Json
            } | ConvertFrom-Json
            
            Format-ClickablePath -Path $pwshProfiles.CurrentUserCurrentHost -Label "Current User, Current Host:"
            Format-ClickablePath -Path $pwshProfiles.CurrentUserAllHosts -Label "Current User, All Hosts:"
            Format-ClickablePath -Path $pwshProfiles.AllUsersCurrentHost -Label "All Users, Current Host:"
            Format-ClickablePath -Path $pwshProfiles.AllUsersAllHosts -Label "All Users, All Hosts:"
        } else {
            Write-Host "  Not installed" -ForegroundColor Gray
        }
    }
    
    Write-Host ""
} #<

# PowerShell Profile Paths Alias (with spaces - no conversion)
function Show-PowerShellProfilesSpaces { #>

    # .SYNOPSIS
    # Displays all PowerShell profile paths for PowerShell 5 and PowerShell 7.
    # Shows original paths with spaces (no junction conversion).

    
    Write-Host "`nPowerShell Profile Paths (Original with Spaces)" -ForegroundColor Cyan
    Write-Host "================================================`n" -ForegroundColor Cyan
    
    $currentPSVersion = $PSVersionTable.PSVersion.Major
    
    # Helper function to format path (no conversion, shows original with spaces)
    function Format-ClickablePathSpaces {
        param([string]$Path, [string]$Label)
        if ($Path) {
            # Output label and path on one line: label in white, path in cyan
            # No path conversion - shows original paths with spaces
            Write-Host "  $Label " -ForegroundColor White -NoNewline
            Write-Host $Path -ForegroundColor Cyan
        } else {
            Write-Host "  $Label " -ForegroundColor White -NoNewline
            Write-Host "(not set)" -ForegroundColor DarkGray
        }
    }
    
    # PowerShell 5 (Windows PowerShell)
    if ($currentPSVersion -eq 5) {
        Write-Host "PowerShell 5 (Windows PowerShell) - Current:" -ForegroundColor Yellow
        Format-ClickablePathSpaces -Path $PROFILE.CurrentUserCurrentHost -Label "Current User, Current Host:"
        Format-ClickablePathSpaces -Path $PROFILE.CurrentUserAllHosts -Label "Current User, All Hosts:"
        Format-ClickablePathSpaces -Path $PROFILE.AllUsersCurrentHost -Label "All Users, Current Host:"
        Format-ClickablePathSpaces -Path $PROFILE.AllUsersAllHosts -Label "All Users, All Hosts:"
    } else {
        Write-Host "PowerShell 5 (Windows PowerShell):" -ForegroundColor Yellow
        if (Get-Command powershell.exe -ErrorAction SilentlyContinue) {
            $ps5Profiles = powershell.exe -NoProfile -Command {
                @{
                    CurrentUserCurrentHost = $PROFILE.CurrentUserCurrentHost
                    CurrentUserAllHosts = $PROFILE.CurrentUserAllHosts
                    AllUsersCurrentHost = $PROFILE.AllUsersCurrentHost
                    AllUsersAllHosts = $PROFILE.AllUsersAllHosts
                } | ConvertTo-Json
            } | ConvertFrom-Json
            
            Format-ClickablePathSpaces -Path $ps5Profiles.CurrentUserCurrentHost -Label "Current User, Current Host:"
            Format-ClickablePathSpaces -Path $ps5Profiles.CurrentUserAllHosts -Label "Current User, All Hosts:"
            Format-ClickablePathSpaces -Path $ps5Profiles.AllUsersCurrentHost -Label "All Users, Current Host:"
            Format-ClickablePathSpaces -Path $ps5Profiles.AllUsersAllHosts -Label "All Users, All Hosts:"
        } else {
            Write-Host "  Not available" -ForegroundColor Gray
        }
    }
    
    Write-Host ""
    
    # PowerShell 7 (pwsh)
    if ($currentPSVersion -ge 7) {
        Write-Host "PowerShell 7 (pwsh) - Current:" -ForegroundColor Yellow
        Format-ClickablePathSpaces -Path $PROFILE.CurrentUserCurrentHost -Label "Current User, Current Host:"
        Format-ClickablePathSpaces -Path $PROFILE.CurrentUserAllHosts -Label "Current User, All Hosts:"
        Format-ClickablePathSpaces -Path $PROFILE.AllUsersCurrentHost -Label "All Users, Current Host:"
        Format-ClickablePathSpaces -Path $PROFILE.AllUsersAllHosts -Label "All Users, All Hosts:"
    } else {
        Write-Host "PowerShell 7 (pwsh):" -ForegroundColor Yellow
        if (Get-Command pwsh -ErrorAction SilentlyContinue) {
            $pwshProfiles = pwsh -NoProfile -Command {
                @{
                    CurrentUserCurrentHost = $PROFILE.CurrentUserCurrentHost
                    CurrentUserAllHosts = $PROFILE.CurrentUserAllHosts
                    AllUsersCurrentHost = $PROFILE.AllUsersCurrentHost
                    AllUsersAllHosts = $PROFILE.AllUsersAllHosts
                } | ConvertTo-Json
            } | ConvertFrom-Json
            
            Format-ClickablePathSpaces -Path $pwshProfiles.CurrentUserCurrentHost -Label "Current User, Current Host:"
            Format-ClickablePathSpaces -Path $pwshProfiles.CurrentUserAllHosts -Label "Current User, All Hosts:"
            Format-ClickablePathSpaces -Path $pwshProfiles.AllUsersCurrentHost -Label "All Users, Current Host:"
            Format-ClickablePathSpaces -Path $pwshProfiles.AllUsersAllHosts -Label "All Users, All Hosts:"
        } else {
            Write-Host "  Not installed" -ForegroundColor Gray
        }
    }
    
    Write-Host ""
} #<

# Create aliases
Set-Alias -Name Show-Profiles -Value Show-PowerShellProfiles
Set-Alias -Name profiles -Value Show-PowerShellProfiles
Set-Alias -Name show-profiles-spaces -Value Show-PowerShellProfilesSpaces





$env:UV_TOOL_DIR = "D:\_dev\.AppData\uv\tools"
