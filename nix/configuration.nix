{
  modulesPath,
  lib,
  pkgs,
  ...
}@args:
{
  imports = [
    (modulesPath + "/installer/scan/not-detected.nix")
    (modulesPath + "/profiles/qemu-guest.nix")
    ./disk-config.nix
    ./bedrock.nix
    ./server.nix
  ];

  networking = {
    hostName = "dan-sansouci";
    # networkmanager.enable = true;
    # firewall.allowedTCPPorts = [ 3000 ];
  };

  #Sops Stuff
  sops.defaultSopsFile = ./secrets/secrets.yaml;
  sops.age.keyFile = "/var/lib/sops-nix/key.txt";
  sops.age.generateKey = false;

  # # All secrets
  # sops.secrets.DATABASE_URL = { };
  # sops.secrets.TCP_ADDR = { };
  # sops.secrets.S3_BUCKET_NAME = { };
  # sops.secrets.S3_API_TOKEN = { };
  # sops.secrets.S3_ACCOUNT_ID = { };
  # sops.secrets.S3_ACCESS_KEY_ID = { };
  # sops.secrets.S3_ACCESS_KEY_SECRET = { };
  # sops.secrets.S3_API_ENDPOINT = { };
  # sops.secrets.DUMMY_DIR = { };

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
    pkgs.rust-bin.stable.latest.default
    pkgs.gcc
  ];

  nix.settings = {
    auto-optimise-store = true;
    # download-buffer-size = 524288000; # 500 MiB
    experimental-features = [
      "nix-command"
      "flakes"
    ];
  };

  users.users.joncorv = {
    isNormalUser = true;
    extraGroups = [
      "wheel"
      "networkmanager"
    ];
    # packages = with pkgs; [
    #   tree
    # ];
  };

  # Temp password only used for new-installs
  # users.users.root.initialPassword = "1234";

  # Only allow connections from my verified users.
  # These are Public Keys and can be commited to version control.
  users.users.root.openssh.authorizedKeys.keys = [
    "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIFQqcoRpf/3hi+RZtrsx3V/mgp8xOQ+Q+tNfE3PLXEyl"
    "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIEYi1z8oaqJLZyUd+jh1TI323ThXJPljm7hGgDBmNFp+"
    "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIC//rZ7+MmwQ6Bb1iXRGkVzt+FtRLDR1aDnpFwQkI4hw big-sky"
    "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAICo9vjJUdKr1RBtVy4J7wmxfxr+dLI3lwkJzwKvGLeK4 joncorv@SN4622128972"
  ]
  ++ (args.extraPublicKeys or [ ]); # this is used for unit-testing this module and can be removed if not needed

  boot.loader.grub = {
    efiSupport = true;
    efiInstallAsRemovable = true;
  };

  # WARNING: SERIOUSLY... DO NOT TOUCH
  system.stateVersion = "24.05";
}
