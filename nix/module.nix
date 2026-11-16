self:
{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.services.oxevu;
in
{
  options.services.oxevu = {
    enable = lib.mkEnableOption "Oxevu media server";
    package = lib.mkOption {
      type = lib.types.package;
      default = self.packages.${pkgs.stdenv.hostPlatform.system}.default;
    };
    user = lib.mkOption {
      type = lib.types.str;
      default = "oxevu";
    };
    group = lib.mkOption {
      type = lib.types.str;
      default = "oxevu";
    };
    dataDir = lib.mkOption {
      type = lib.types.str;
      default = "/var/lib/oxevu";
    };
    bind = lib.mkOption {
      type = lib.types.str;
      default = "127.0.0.1:3000";
    };
    origin = lib.mkOption {
      type = lib.types.str;
      default = "http://localhost:3000";
    };
    mediaDirectories = lib.mkOption {
      type = lib.types.listOf lib.types.str;
      default = [ ];
    };
  };
  config = lib.mkIf cfg.enable {
    users.users = lib.mkIf (cfg.user == "oxevu") {
      oxevu = {
        isSystemUser = true;
        group = cfg.group;
      };
    };
    users.groups = lib.mkIf (cfg.group == "oxevu") { oxevu = { }; };
    systemd.tmpfiles.rules = [ "d ${cfg.dataDir} 0700 ${cfg.user} ${cfg.group} -" ];
    systemd.services.oxevu = {
      description = "Oxevu media server";
      wantedBy = [ "multi-user.target" ];
      after = [ "network.target" ];
      unitConfig.RequiresMountsFor = cfg.mediaDirectories ++ [ cfg.dataDir ];
      environment = {
        OXEVU_DATA_DIR = cfg.dataDir;
        OXEVU_BIND = cfg.bind;
        OXEVU_ORIGIN = cfg.origin;
      };
      serviceConfig = {
        ExecStart = "${cfg.package}/bin/oxevu-server";
        User = cfg.user;
        Group = cfg.group;
        Restart = "on-failure";
        UMask = "0077";
        NoNewPrivileges = true;
        ProtectSystem = "strict";
        ReadWritePaths = [ cfg.dataDir ];
        ReadOnlyPaths = cfg.mediaDirectories;
        PrivateTmp = true;
      };
    };
  };
}
