# Example NixOS configuration for Hex Chess deployment
# Add this to your /etc/nixos/configuration.nix

{ config, pkgs, ... }:

{
  imports = [
    ./hex-chess.nix
  ];

  # Enable the Hex Chess service
  services.hex-chess = {
    enable = true;
    port = 3001;
    host = "0.0.0.0";
  };

  # Optional: Add to your existing nginx configuration
  services.nginx = {
    enable = true;
    # Your existing nginx config...
  };

  # Optional: Firewall configuration
  networking.firewall = {
    enable = true;
    allowedTCPPorts = [ 80 443 3001 ];
  };

  # Optional: SSL certificates
  security.acme = {
    acceptTerms = true;
    defaults.email = "your-email@example.com";
  };
}
