#!/usr/bin/env bash

set -ex

REPO=${REPO:-https://github.com/Defelo/sandkasten.git}
DISK=${1:-/dev/sda}
ROOT=${2:-${DISK}1}

mkdir -p ~/.config/nix
cat << EOF > ~/.config/nix/nix.conf
extra-experimental-features = nix-command flakes
extra-substituters = https://sandkasten.cachix.org
extra-trusted-public-keys = sandkasten.cachix.org-1:Pa7qfdlx7bZkko+ojaaEG9pyziZkaru9v4TfcioqNZw=
EOF

nix profile install nixpkgs#git

echo -ne 'o\nn\n\n\n\n\na\nw\n' | fdisk $DISK
mkfs.ext4 $ROOT
mount $ROOT /mnt
mkdir /mnt/root
git clone "$REPO" /mnt/root/sandkasten
nixos-install --flake /mnt/root/sandkasten#sandkasten --no-channel-copy --no-root-password
reboot
