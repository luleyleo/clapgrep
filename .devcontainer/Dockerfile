FROM docker.io/archlinux:base

ARG USERNAME=dev
ARG USER_UID=1000
ARG USER_GID=$USER_UID

# Create the user
RUN groupadd --gid $USER_GID $USERNAME \
    && useradd --uid $USER_UID --gid $USER_GID -m $USERNAME

# Update the container and install all packages we need for devcontainer in general
# This is seperated from the actual project specific packages for readability
RUN pacman -Syu --noconfirm && \
    pacman -S --noconfirm sudo git less nftables && \
    # Then we install the project-specific packages, so the ones we need want for development
    # Independent of the devcontainer. Finally we clear the pacman cache
    pacman -S --noconfirm just meson pkgconf appstream rust rust-analyzer gcc libglvnd gtk4 libadwaita gtksourceview5 poppler-glib blueprint-compiler && \
    pacman -Sc --noconfirm

# And copy our startup script that needs to run after the container got started
COPY --chown=root:root --chmod=755 ./startup.sh /startup.sh

USER $USERNAME
