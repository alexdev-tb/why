# why - shell hook for Zsh
# Add to ~/.zshrc: eval "$(why --hook zsh)"

[[ -n "$_WHY_HOOK_LOADED" ]] && return
_WHY_HOOK_LOADED=1

autoload -Uz add-zsh-hook

_WHY_STDERR_FILE="${TMPDIR:-/tmp}/why_stderr_$$"

trap 'rm -f "$_WHY_STDERR_FILE"' EXIT

_why_preexec() {
    export WHY_LAST_CMD="$1"
}
add-zsh-hook preexec _why_preexec

_why_precmd() {
    local last_exit=$?
    export WHY_LAST_EXIT=$last_exit

    if [[ $last_exit -ne 0 ]] && [[ -s "$_WHY_STDERR_FILE" ]]; then
        export WHY_LAST_STDERR=$(cat "$_WHY_STDERR_FILE")
    else
        export WHY_LAST_STDERR=""
    fi
}
add-zsh-hook precmd _why_precmd

why-run() {
    local cmd="$*"
    eval "$cmd" 2> >(tee "$_WHY_STDERR_FILE" >&2)
    local exit_code=$?

    export WHY_LAST_EXIT=$exit_code
    export WHY_LAST_CMD="$cmd"

    if [[ $exit_code -ne 0 ]] && [[ -f "$_WHY_STDERR_FILE" ]]; then
        export WHY_LAST_STDERR=$(cat "$_WHY_STDERR_FILE")
    else
        export WHY_LAST_STDERR=""
    fi

    return $exit_code
}
