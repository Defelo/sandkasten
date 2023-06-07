#!/usr/bin/env bash

set -ex

FLAKE=${FLAKE:-github:Defelo/sandkasten#vm}
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
nix flake new --template "$FLAKE" /mnt/root/sandkasten

nixos-generate-config --root /mnt --show-hardware-config > /mnt/root/sandkasten/hardware-configuration.nix
sed -i -E "s#boot.loader.grub.device = \"/dev/sda\";#boot.loader.grub.device = \"$DISK\";#" /mnt/root/sandkasten/configuration.nix

nixos-install --flake /mnt/root/sandkasten#sandkasten --no-channel-copy --no-root-password

reboot
