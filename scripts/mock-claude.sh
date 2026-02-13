#!/bin/bash
# Mock Claude Code CLI for demo recording via VHS

# Emit OSC 7 for CWD tracking
report_cwd() {
    printf '\e]7;file://%s%s\e\\' "$(hostname)" "$PWD"
}

# Print initial header
print_startup() {
    local cwd_display="${PWD/#$HOME/~}"
    printf '\e[2J\e[H'
    printf '\n'
    printf '  \e[38;5;208m✻\e[0m  \e[1m%s\e[0m\n' "$cwd_display"
    printf '     \e[38;5;34mOpus 4.6 is here\e[0m \e[2m- /extra-usage to enable\e[0m\n'
    printf '\n'
}

# Print Claude-style tool use block
claude_tool_use() {
    local explanation="$1"
    local command="$2"
    printf '\n'
    printf '  %s\n' "$explanation"
    printf '\n'
    printf '  \e[90m$\e[0m %s\n' "$command"
    printf '  \e[90m⎿\e[0m (No output)\n'
    printf '\n'
}

# Handle natural language input for demo scenarios
handle_input() {
    local input="$1"

    case "$input" in
        *src\ directory*|*src\ dir*|*look*src*)
            claude_tool_use \
                "I'll navigate to the src directory to explore the source code." \
                "cd src"
            cd src 2>/dev/null && report_cwd
            ;;
        *tree\ module*|*explore*tree*|*tree\ dir*)
            claude_tool_use \
                "Let me explore the tree module." \
                "cd tree"
            cd tree 2>/dev/null && report_cwd
            ;;
        *project\ root*|*go\ back*|*back\ to*root*)
            claude_tool_use \
                "I'll go back to the project root." \
                "cd $(python3 -c "import os.path; print(os.path.relpath('$PROJECT_ROOT','$PWD'))" 2>/dev/null || echo '..')"
            cd "$PROJECT_ROOT" 2>/dev/null && report_cwd
            ;;
        cd\ *)
            local target="${input#cd }"
            if cd "$target" 2>/dev/null; then
                printf '  \e[90m⎿\e[0m (No output)\n\n'
                report_cwd
            else
                printf '  \e[90m⎿\e[0m cd: no such directory: %s\n\n' "$target"
            fi
            ;;
        *)
            if [[ -n "$input" ]]; then
                printf '\n  Sure, I can help with that.\n\n'
            fi
            ;;
    esac
}

# --- Main ---
PROJECT_ROOT="$PWD"
print_startup
report_cwd

while true; do
    printf '  \e[1;35m>\e[0m '
    read -r input || break
    handle_input "$input"
done
