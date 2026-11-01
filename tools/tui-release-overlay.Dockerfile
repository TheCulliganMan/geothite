# Additive, verified browser AND terminal release. Preserve the pinned base's
# graphical client, server, packs and Flygon assets. Generated context is ignored.
ARG TUI_BASE_IMAGE=geothite:tui-base
FROM ${TUI_BASE_IMAGE}
COPY --chown=65532:65532 tui/ /srv/crystal/web/tui/
COPY --chown=65532:65532 install.sh /srv/crystal/web/install.sh
COPY --chown=65532:65532 downloads/ /srv/crystal/web/downloads/
