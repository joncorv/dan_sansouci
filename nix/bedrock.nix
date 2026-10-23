{ pkgs, config, ... }:
let
  toolchain = pkgs.rust-bin.stable.latest.minimal;
  bedrock = pkgs.callPackage ../bedrock/package.nix {
    rustPlatform = pkgs.makeRustPlatform {
      cargo = toolchain;
      rustc = toolchain;
    };

  };
in
{

  # All secrets
  sops.secrets.DATABASE_URL = { };
  sops.secrets.TCP_ADDR = { };
  sops.secrets.S3_BUCKET_NAME = { };
  sops.secrets.S3_API_TOKEN = { };
  sops.secrets.S3_ACCOUNT_ID = { };
  sops.secrets.S3_ACCESS_KEY_ID = { };
  sops.secrets.S3_ACCESS_KEY_SECRET = { };
  sops.secrets.S3_API_ENDPOINT = { };
  sops.secrets.DUMMY_DIR = { };

  sops.templates."bedrock.env" = {
    content = ''
      DATABASE_URL=${config.sops.placeholder.DATABASE_URL}
      TCP_ADDR=${config.sops.placeholder.TCP_ADDR}
      S3_BUCKET_NAME=${config.sops.placeholder.S3_BUCKET_NAME}
      S3_API_TOKEN=${config.sops.placeholder.S3_API_TOKEN}
      S3_ACCOUNT_ID=${config.sops.placeholder.S3_ACCOUNT_ID}
      S3_ACCESS_KEY_ID=${config.sops.placeholder.S3_ACCESS_KEY_ID}
      S3_ACCESS_KEY_SECRET=${config.sops.placeholder.S3_ACCESS_KEY_SECRET}
      S3_API_ENDPOINT=${config.sops.placeholder.S3_API_ENDPOINT}
      DUMMY_DIR=${config.sops.placeholder.DUMMY_DIR}
    '';
    restartUnits = [ "bedrock.service" ];
  };

  systemd.services.bedrock = {
    description = "Dan Sansouci's Bedrock";
    wantedBy = [ "multi-user.target" ];
    after = [ "network-online.target" ];
    wants = [ "network-online.target" ];

    serviceConfig = {
      ExecStart = "${bedrock}/bin/bedrock";
      Restart = "on-failure";
      DynamicUser = true;
      StateDirectory = "bedrock";
      EnvironmentFile = config.sops.templates."bedrock.env".path;
    };

    environment = {
      RUST_LOG = "info";
    };
  };
}
