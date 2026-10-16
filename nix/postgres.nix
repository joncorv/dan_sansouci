{
  lib,
  pkgs,
  config,
  ...
}: 

{
  config.services.postgresql = {
    enable = true;
    ensureDatabases = [ "cellar" ];
    authentication = pkgs.lib.mkOverride 10 ''
      #type database  DBuser  auth-method
      local all       all     trust
    '';
  };
}
