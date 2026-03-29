# why - shell hook for Zsh
# Add to ~/.zshrc: eval "$(why --hook zsh)"

[[ -n "$_WHY_HOOK_LOADED" ]] && return
_WHY_HOOK_LOADED=1

autoload -Uz add-zsh-hook

_WHY_STDERR_FILE="${TMPDIR:-/tmp}/why_stderr_$$"
_WHY_CAPTURING=0
_WHY_STDERR_FD=""

trap 'rm -f "$_WHY_STDERR_FILE"' EXIT

_why_preexec() {
    [[ "$_WHY_CAPTURING" -eq 1 ]] && return
    _WHY_CAPTURING=1
    export WHY_LAST_CMD="$1"
    : > "$_WHY_STDERR_FILE"
    exec {_WHY_STDERR_FD}>&2
    exec 2> >(tee -a "$_WHY_STDERR_FILE" >&$_WHY_STDERR_FD)
}
add-zsh-hook preexec _why_preexec

_why_precmd() {
    local last_exit=$?

    if [[ "$_WHY_CAPTURING" -eq 1 ]]; then
        exec 2>&$_WHY_STDERR_FD
        exec {_WHY_STDERR_FD}>&-
        _WHY_STDERR_FD=""
        _WHY_CAPTURING=0
        # Let the process substitution drain without blocking the prompt.
        sleep 0.01
    fi

    export WHY_LAST_EXIT=$last_exit

    if [[ $last_exit -ne 0 ]] && [[ -s "$_WHY_STDERR_FILE" ]]; then
        export WHY_LAST_STDERR=$(< "$_WHY_STDERR_FILE")
    else
        export WHY_LAST_STDERR=""
    fi
}
add-zsh-hook precmd _why_precmd
