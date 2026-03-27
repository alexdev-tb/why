# why - shell hook for Bash
# Add to ~/.bashrc: eval "$(why --hook bash)"

[[ -n "$_WHY_HOOK_LOADED" ]] && return
_WHY_HOOK_LOADED=1

_WHY_STDERR_FILE="${TMPDIR:-/tmp}/why_stderr_$$"
_WHY_CAPTURING=0

trap 'rm -f "$_WHY_STDERR_FILE"' EXIT

# Save original stderr to fd 9
exec 9>&2

_why_preexec() {
    [[ "$_WHY_CAPTURING" -eq 1 ]] && return
    _WHY_CAPTURING=1
    > "$_WHY_STDERR_FILE"
    exec 2> >(tee -a "$_WHY_STDERR_FILE" >&9)
}

_why_prompt_command() {
    local last_exit=$?

    if [[ "$_WHY_CAPTURING" -eq 1 ]]; then
        # Restore stderr to original fd — this closes the pipe to tee
        exec 2>&9
        _WHY_CAPTURING=0
        # Wait for tee to finish flushing to the file
        sleep 0.01
    fi

    export WHY_LAST_EXIT=$last_exit
    export WHY_LAST_CMD=$(HISTTIMEFORMAT= history 1 | sed 's/^[ ]*[0-9]*[ ]*//')

    if [[ $last_exit -ne 0 ]] && [[ -s "$_WHY_STDERR_FILE" ]]; then
        export WHY_LAST_STDERR=$(< "$_WHY_STDERR_FILE")
    else
        export WHY_LAST_STDERR=""
    fi
}

trap '_why_preexec' DEBUG

if [[ -z "$PROMPT_COMMAND" ]]; then
    PROMPT_COMMAND="_why_prompt_command"
else
    PROMPT_COMMAND="_why_prompt_command;${PROMPT_COMMAND}"
fi
