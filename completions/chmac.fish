set -l commands reset set random get perm help inames completions


function __fish_chmac_help_subcommand_completion
    set -l commands reset set random get perm help inames completions
    set -l cmd_args (commandline -opc)

    if test (count $cmd_args) -eq 2
        echo $commands 2>/dev/null | tr " " "\n" || echo ""
    end
end

function __fish_chmac_complete_interfaces
    chmac inames 2>/dev/null || echo ""
end

complete -c chmac -f

complete -c chmac -n "not __fish_seen_subcommand_from $commands" -a reset -d 'Reset the MAC address to the permaddr of the interface'
complete -c chmac -n "not __fish_seen_subcommand_from $commands" -a set -d 'Set the MAC address to a specified value'
complete -c chmac -n "not __fish_seen_subcommand_from $commands" -a random -d 'Set a random MAC address'
complete -c chmac -n "not __fish_seen_subcommand_from $commands" -a get -d 'Get current MAC address'
complete -c chmac -n "not __fish_seen_subcommand_from $commands" -a perm -d 'Get permanent MAC address'
complete -c chmac -n "not __fish_seen_subcommand_from $commands" -a inames -d 'Get a list of all available interfaces'
complete -c chmac -n "not __fish_seen_subcommand_from $commands" -a completions -d 'Print shell completion definitions'
complete -c chmac -n "not __fish_seen_subcommand_from $commands" -a help -d 'Print this message or the help of the given subcommand(s)'

# chmac completions
complete -c chmac -n "not __fish_seen_subcommand_from help; and __fish_seen_subcommand_from completions; and __fish_is_nth_token 2" -a fish


# chmac help
complete -c chmac -f -n "__fish_seen_subcommand_from help" -a "(__fish_chmac_help_subcommand_completion)"

# chmac random | reset | get | perm | set 
complete -c chmac -n "__fish_seen_subcommand_from reset random get perm set; and not __fish_seen_subcommand_from help; and __fish_is_nth_token 2" -ka '(__fish_chmac_complete_interfaces)'

# chmac inames
complete -c chmac -n "__fish_seen_subcommand_from inames; and not __fish_seen_subcommand_from help" -s 1 -l single-line -d 'List all interfaces in a single line rather than one on each line'
