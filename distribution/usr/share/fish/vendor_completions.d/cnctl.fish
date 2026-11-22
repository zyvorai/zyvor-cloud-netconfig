# SPDX-License-Identifier: LicenseRef-Zyvor-Production-2.0
# fish completion for cnctl

# Disable file completions by default
complete -c cnctl -f

# Commands
complete -c cnctl -n "__fish_use_subcommand" -a status -d "Show status information"
complete -c cnctl -n "__fish_use_subcommand" -a apply -d "Apply configuration from file"
complete -c cnctl -n "__fish_use_subcommand" -a reload -d "Reload daemon configuration"
complete -c cnctl -n "__fish_use_subcommand" -a version -d "Show daemon version"
complete -c cnctl -n "__fish_use_subcommand" -a help -d "Show help information"

# status subcommand
complete -c cnctl -n "__fish_seen_subcommand_from status" -a "system network all"

# apply subcommand
complete -c cnctl -n "__fish_seen_subcommand_from apply" -s c -l config -d "Configuration file path" -r -F
complete -c cnctl -n "__fish_seen_subcommand_from apply" -s d -l dry-run -d "Dry-run mode (validate only)"
complete -c cnctl -n "__fish_seen_subcommand_from apply" -s h -l help -d "Show help information"

# reload subcommand
complete -c cnctl -n "__fish_seen_subcommand_from reload" -s f -l force -d "Force reload even if config hasn't changed"
complete -c cnctl -n "__fish_seen_subcommand_from reload" -s h -l help -d "Show help information"
