# Nixos configured for remote deployment

To install using nixos-anywhere on any computer with nix installed, and flakes enabled:
replace user@ip-address with your server credentials
run from LOCAL MACHINE. Nixos-anywhere will connect via SSH on your behalf to the target-host.

```bash
nix run github:nix-community/nixos-anywhere -- \
--kexec-extra-flags "--kexec-syscall" \
--flake .#big-sky --generate-hardware-config nixos-generate-config ./hardware-configuration.nix \
user@ip-address
```

Then to remote in and rebuild / update the system

```bash
nix run nixpkgs#nixos-rebuild -- \
switch \
--no-reexec \
--flake .#big-sky \
--target-host user@ip-address \
--build-host user@ip-address
```

Run software updates:

```bash
nix flake update && \
nix run nixpkgs#nixos-rebuild -- \
switch \
--no-reexec \
--flake .#big-sky \
--target-host user@ip-address \
--build-host user@ip-address
```
