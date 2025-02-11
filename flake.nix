{
  description = "netns-exec - run a process in a Linux network namespace";

  inputs = {
    naersk.url = "github:nix-community/naersk/master";
    utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs = { self, nixpkgs, utils, naersk}:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {inherit system; };
        naersk-lib = pkgs.callPackage naersk { };
      in {
        defaultPackage = 
        pkgs.symlinkJoin {
          name = "netns-exec";
          paths = [
            (naersk-lib.buildPackage ./.)
            (pkgs.writeTextFile {
              name = "netns-exec-completion";
              destination = "/share/bash-completion/completions/netns-exec";
              text = ''
                _comp_cmd_ip__netns()
                {
                    local unquoted
                    _comp_split -l unquoted "$(
                        {
                            ''${1-ip} -c=never netns list 2>/dev/null || ''${1-ip} netns list
                        } | command sed -e 's/ (.*//'
                    )"
                    # namespace names can have spaces, so we quote all of them if needed
                    local ns quoted=()
                    for ns in "''${unquoted[@]}"; do
                        local namespace
                        printf -v namespace '%q' "$ns"
                        quoted+=("$namespace")
                    done
                    ((''${#quoted[@]})) && _comp_compgen -- -W '"''${quoted[@]}"'
                }

                _netns-exec()
                {
                    local cur
                    cur="''${COMP_WORDS[$COMP_CWORD]}"
                    if [[ $COMP_CWORD == 1 ]] ; then
                        _comp_cmd_ip__netns ip
                        return 0
                    fi
                    if [[ $COMP_CWORD -ge 2 ]] ; then
                        _comp_command_offset 2
                        return 0
                    fi
                }

                complete -F _netns-exec netns-exec
              '';
            })
          ];
        };
        
        devShell = with pkgs; mkShell {
          buildInputs = [ rustc cargo rustPackages.clippy rustfmt bashInteractive ];
          RUST_SRC_PATH = rustPlatform.rustLibSrc;
        };
      }
    ) 
  ;
}
