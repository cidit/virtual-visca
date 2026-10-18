{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-26.05";
  };

  outputs = { self , nixpkgs ,... }: let
    # system should match the system you are running on
    system = "x86_64-linux";
  in {
    devShells."${system}".default = let
      pkgs = import nixpkgs { inherit system; };
      buildInputs = with pkgs; [
        udev alsa-lib vulkan-loader
        libxkbcommon wayland # To use the wayland feature
      ];
    in pkgs.mkShell {
      # create an environment with nodejs_24, pnpm, and yarn
      # packages = with pkgs; [
      #   nodejs_24
      #   pnpm
      #   (yarn.override { nodejs = nodejs_24; })
      #   nushell
      # ];

      # shellHook = ''
      #   echo "node `node --version`"
      #   exec nu
      # '';

      inherit buildInputs;
      packages = with pkgs; [
        pkg-config
      ];


      LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
    };
  };
}
