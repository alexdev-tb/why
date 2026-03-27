# why - shell hook for Zsh
# Add to ~/.zshrc: eval "$(why --hook zsh)"

[[ -n "$_WHY_HOOK_LOADED" ]] && return
_WHY_HOOK_LOADED=1

autoload -Uz add-zsh-hook

_WHY_STDERR_FILE="${TMPDIR:-/tmp}/why_stderr_$$"

trap 'rm -f "$_WHY_STDERR_FILE"' EXIT

# Save original stderr to fd 9
exec 9>&2

_why_preexec() {
    export WHY_LAST_CMD="$1"
    > "$_WHY_STDERR_FILE"
    exec 2> >(tee -a "$_WHY_STDERR_FILE" >&9)
}
add-zsh-hook preexec _why_preexec

_why_precmd() {
    local last_exit=$?
    exec 2>&9
    # Wait for tee to finish flushing to the file
    sleep 0.01
    export WHY_LAST_EXIT=$last_exit

    if [[ $last_exit -ne 0 ]] && [[ -s "$_WHY_STDERR_FILE" ]]; then
        export WHY_LAST_STDERR=$(< "$_WHY_STDERR_FILE")
    else
        export WHY_LAST_STDERR=""
    fi
}
add-zsh-hook precmd _why_precmd
