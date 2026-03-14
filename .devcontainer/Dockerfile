FROM docker.io/archlinux:base

ARG USERNAME=dev
ARG USER_UID=1000
ARG USER_GID=$USER_UID

# Create the user
RUN groupadd --gid $USER_GID $USERNAME \
    && useradd --uid $USER_UID --gid $USER_GID -m $USERNAME

# Required for devcontainer
RUN pacman -Syu --noconfirm && \
    pacman -S --noconfirm sudo git less nftables && \
    pacman -Sc --noconfirm

# Required for Clapgrep
RUN pacman -Syu --noconfirm && \
    pacman -S --noconfirm \
        flatpak-builder libglvnd \
        just meson pkgconf appstream \
        rust rust-analyzer gcc blueprint-compiler \
        gtk4 libadwaita gtksourceview5 poppler-glib && \
    pacman -Sc --noconfirm

USER $USERNAME
