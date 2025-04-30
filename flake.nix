{
  description = "Development shell for baseband_fuzz";
  inputs.nixpkgs.url = "github:NixOS/nixpkgs";
  outputs = {
    self,
    nixpkgs,
  }: {
    devShells.x86_64-linux.default = let
      pkgs = import nixpkgs {
        system = "x86_64-linux";
        config = {
          permittedInsecurePackages = [
            "openssl-1.1.1w"
            "python-2.7.18.8-env"
            "python-2.7.18.8"
          ];
        };
      };
      llvmPkgs = pkgs.llvmPackages;
      python27Packages = pkgs.python27.pkgs;
      oldPyOpenSSL = python27Packages.pyopenssl.overridePythonAttrs (old: {
        version = "22.0.0"; # Last version compatible with Python 2.7
        src = pkgs.fetchPypi {
          pname = "pyOpenSSL";
          version = "22.0.0";
          sha256 = "sha256-/CTTWuk7g6aCKOKyEKrSx7MMO1uOJlKLcWY6BYyDFcE=";
        };
      });

      python27WithSsl = pkgs.python27.withPackages (ps: [
        oldPyOpenSSL
      ]);
      deps = with pkgs; [
        rustc
        cargo
        ncurses5
        libcxx
        pixman
        socat
        glib
        meson
        python27WithSsl # Use the enhanced Python instead
        ninja
        pkg-config
        flex
        bison
        openssl_1_1
        llvm
        llvmPkgs.clang
        llvmPkgs.libclang
        llvmPkgs.libcxxClang
      ];
    in
      pkgs.mkShell {
        buildInputs = deps;
        shellHook = ''
          export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${builtins.toString (pkgs.lib.makeLibraryPath deps)}:${pkgs.openssl_1_1}/lib";
          export LIBCLANG_PATH="${builtins.toString llvmPkgs.libclang.lib}/lib";
          export PATH="$PATH:${pkgs.rustc}/bin:${pkgs.cargo}/bin";
          export PYTHONPATH="$PYTHONPATH:${pkgs.python27Packages.pyopenssl}/${pkgs.python27Full.sitePackages}";
          # Make sure Python can find OpenSSL
          export SSL_CERT_FILE="${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt";
          export OPENSSL_CONF="${pkgs.openssl_1_1.out}/etc/ssl/openssl.cnf";
        '';
      };
  };
}
