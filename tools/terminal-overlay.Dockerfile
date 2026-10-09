# Add verified, ignored prebuilt releases to an explicitly pinned live image.
# Never put game content inside this build context or release downloads.
ARG TERMINAL_BASE_IMAGE
FROM ${TERMINAL_BASE_IMAGE}
COPY --chown=65532:65532 install.sh /srv/crystal/web/install.sh
COPY --chown=65532:65532 downloads/ /srv/crystal/web/downloads/
