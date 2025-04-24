{
  description = "Development shell with Python, Go, and Protobuf tools";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs";

  outputs = {
    self,
    nixpkgs,
  }: {
    devShells.x86_64-linux.default = let
      pkgs = import nixpkgs {system = "x86_64-linux";};
      deps = with pkgs; [
      ];
    in
      pkgs.mkShell {
        buildInputs = deps;
        shellHook = ''
          export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${builtins.toString (pkgs.lib.makeLibraryPath deps)}";
        '';
      };
  };
}
