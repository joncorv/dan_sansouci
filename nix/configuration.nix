{
  modulesPath,
  lib,
  pkgs,
  ...
} @ args:
{
  imports = [
    (modulesPath + "/installer/scan/not-detected.nix")
    (modulesPath + "/profiles/qemu-guest.nix")
    ./disk-config.nix
    ./postgres.nix
  ];

  networking = {
    hostName = "big-sky";
    # networkmanager.enable = true;
    # firewall.allowedTCPPorts = [ 3000 ];
  };

  boot.loader.grub = {
    efiSupport = true;
    efiInstallAsRemovable = true;
  };

  # SSH
  services.openssh.enable = true;
  services.openssh.settings = {
    PasswordAuthentication = false;
    KbdInteractiveAuthentication = false;
    PermitRootLogin = "prohibit-password";
  };

  # Tailscale
  services.tailscale.enable = true;
  networking.firewall.interfaces."tailscale0".allowedTCPPorts = [ 22 ];

  nixpkgs.config.allowUnfree = true;
  environment.systemPackages = map lib.lowPrio [
    pkgs.curl
    pkgs.gitMinimal
    pkgs.neovim
    pkgs.gh
    pkgs.ghostty.terminfo
  ];

  # Temp password only used for new-installs
  # users.users.root.initialPassword = "1234";

  # Only allow connections from my verified users.
  # These are Public Keys and can be commited to version control.
  users.users.root.openssh.authorizedKeys.keys =
  [
    "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIFQqcoRpf/3hi+RZtrsx3V/mgp8xOQ+Q+tNfE3PLXEyl"
    "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIEYi1z8oaqJLZyUd+jh1TI323ThXJPljm7hGgDBmNFp+"
    "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIC//rZ7+MmwQ6Bb1iXRGkVzt+FtRLDR1aDnpFwQkI4hw big-sky"
  ] ++ (args.extraPublicKeys or []); # this is used for unit-testing this module and can be removed if not needed

  # WARNING: SERIOUSLY... DO NOT TOUCH
  system.stateVersion = "24.05";
}
