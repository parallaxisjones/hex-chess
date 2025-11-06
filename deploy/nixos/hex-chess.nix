{ config, lib, pkgs, ... }:

with lib;

let
  cfg = config.services.hex-chess;
  hexChessPkg = pkgs.hex-chess-game;
  signalingPkg = pkgs.hex-chess-signaling;
in
{
  options.services.hex-chess = {
    enable = mkEnableOption "Hex Chess game service";
    
    port = mkOption {
      type = types.port;
      default = 3001;
      description = "Port for the signaling server";
    };
    
    host = mkOption {
      type = types.str;
      default = "0.0.0.0";
      description = "Host to bind the signaling server to";
    };
    
    webRoot = mkOption {
      type = types.path;
      default = hexChessPkg;
      description = "Path to the web assets";
    };
  };

  config = mkIf cfg.enable {
    # Nginx configuration for serving the web app
    services.nginx = {
      enable = true;
      virtualHosts."hex-chess.local" = {
        root = cfg.webRoot;
        index = "index.html";
        
        # Serve static files
        locations."/" = {
          tryFiles = "$uri $uri/ /index.html";
        };
        
        # WebSocket proxy for signaling server
        locations."/ws" = {
          proxyPass = "http://127.0.0.1:${toString cfg.port}";
          proxyWebsockets = true;
          proxySetHeaders = {
            Host = "$host";
            X-Real-IP = "$remote_addr";
            X-Forwarded-For = "$proxy_add_x_forwarded_for";
            X-Forwarded-Proto = "$scheme";
          };
        };
        
        # API proxy for signaling server
        locations."/api/" = {
          proxyPass = "http://127.0.0.1:${toString cfg.port}/";
          proxySetHeaders = {
            Host = "$host";
            X-Real-IP = "$remote_addr";
            X-Forwarded-For = "$proxy_add_x_forwarded_for";
            X-Forwarded-Proto = "$scheme";
          };
        };
        
        # Enable gzip compression
        extraConfig = ''
          gzip on;
          gzip_vary on;
          gzip_min_length 1024;
          gzip_proxied any;
          gzip_comp_level 6;
          gzip_types
            text/plain
            text/css
            text/xml
            text/javascript
            application/json
            application/javascript
            application/xml+rss
            application/atom+xml
            image/svg+xml;
        '';
      };
    };

    # Systemd service for the signaling server
    systemd.services.hex-chess-signaling = {
      description = "Hex Chess Signaling Server";
      after = [ "network.target" ];
      wantedBy = [ "multi-user.target" ];
      
      serviceConfig = {
        Type = "simple";
        User = "hex-chess";
        Group = "hex-chess";
        ExecStart = "${signalingPkg}/bin/signaling-server";
        Restart = "always";
        RestartSec = 5;
        
        # Security settings
        NoNewPrivileges = true;
        PrivateTmp = true;
        ProtectSystem = "strict";
        ProtectHome = true;
        ReadWritePaths = [ "/var/lib/hex-chess" ];
        
        # Environment
        Environment = [
          "RUST_LOG=info"
          "HOST=${cfg.host}"
          "PORT=${toString cfg.port}"
        ];
      };
    };

    # Create user for the service
    users.users.hex-chess = {
      isSystemUser = true;
      group = "hex-chess";
      home = "/var/lib/hex-chess";
      createHome = true;
    };
    
    users.groups.hex-chess = {};

    # Firewall configuration
    networking.firewall = {
      allowedTCPPorts = [ 80 443 cfg.port ];
    };

    # Optional: Let's Encrypt SSL
    security.acme = {
      acceptTerms = true;
      defaults.email = "admin@example.com";
    };
    
    services.nginx.virtualHosts."hex-chess.local" = {
      enableACME = true;
      forceSSL = true;
    };
  };
}
