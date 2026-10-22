{
  lib,
  pkgs,
  config,
  ...
}:

{

  config = {

    services.caddy = {
      enable = true;
      virtualHosts."api.dansansouci.com".extraConfig = ''
        reverse_proxy 127.0.0.1:8080
      '';
    };

    networking.firewall.allowedTCPPorts = [
      80
      443
    ];

    environment.systemPackages = with pkgs; [
      sqlx-cli
      rsync
      bacon
    ];

    services.postgresql = {
      enable = true;
      package = pkgs.postgresql_15;
      # package = pkgs.postgresql.pg_config;
      ensureDatabases = [ "dan_sansouci_db" ];
      authentication = pkgs.lib.mkOverride 10 ''
        # TYPE DATABASE USER ADDRESS METHOD
        local all all trust
        host all postgres 127.0.0.1/32 scram-sha-256
        host all postgres ::1/128      scram-sha-256
      '';
    };
  };

}
