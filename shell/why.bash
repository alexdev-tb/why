# why - shell hook for Bash
# Add to ~/.bashrc: eval "$(why --hook bash)"

[[ -n "$_WHY_HOOK_LOADED" ]] && return
_WHY_HOOK_LOADED=1

_WHY_STDERR_FILE="${TMPDIR:-/tmp}/why_stderr_$$"

trap 'rm -f "$_WHY_STDERR_FILE"' EXIT

_why_prompt_command() {
    local last_exit=$?
    export WHY_LAST_EXIT=$last_exit
    export WHY_LAST_CMD=$(HISTTIMEFORMAT= history 1 | sed 's/^[ ]*[0-9]*[ ]*//')

    if [[ $last_exit -ne 0 ]] && [[ -s "$_WHY_STDERR_FILE" ]]; then
        export WHY_LAST_STDERR=$(cat "$_WHY_STDERR_FILE")
    else
        export WHY_LAST_STDERR=""
    fi
}

if [[ -z "$PROMPT_COMMAND" ]]; then
    PROMPT_COMMAND="_why_prompt_command"
else
    PROMPT_COMMAND="_why_prompt_command;${PROMPT_COMMAND}"
fi

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
