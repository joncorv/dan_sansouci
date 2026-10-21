{
  lib,
  pkgs,
  config,
  ...
}:

{

  config = {

    environment.systemPackages = with pkgs; [
      sqlx-cli
      rsync
    ];

    services.postgresql = {
      enable = true;
      ensureDatabases = [ "dan_sansouci_db" ];
      authentication = pkgs.lib.mkOverride 10 ''
        #type database  DBuser  auth-method
        local all       all     trust
      '';
    };
  };

}
