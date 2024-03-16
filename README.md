# chmac

`chmac` is a small CLI tool that allows users to change network interfaces MAC address by making `ioctl` calls.

```
Usage: chmac <COMMAND>

Commands:
  reset        Reset the MAC address to the permaddr of the interface
  set          Set the MAC address to a specified value
  random       Set a random MAC address
  get          Get current MAC address
  perm         Get permanent MAC address
  inames       Get a list of all available interfaces
  completions  Print shell completion definitions
  help         Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```
