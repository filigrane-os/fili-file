# Casier (fili-file): build and install.
#
#   make                       build the release binary
#   make test                  unit tests
#   sudo make install          install under /usr/local (PREFIX=/usr for a system package)
#   make install DESTDIR=pkg PREFIX=/usr      stage into a directory (packagers)
#   make install-data PREFIX=~/.local         everything but the binary (launchers, icons, portal, licences, what the shell reads)
#
# Everything the application brings to the Filigrane desktop lives here and is installed together: the program, the launchers,
# the icons, and shell/ (the fallback menu of the bar, the brief of Rob), which ends in $(DATADIR)/filigrane/shell/ where
# `fili-bar link-system` finds it.
#
# cxx-qt finds Qt through qmake. QMAKE overrides the search below (Qt 6 only), for example
#   make QMAKE=/opt/qt6/bin/qmake6
PREFIX ?= /usr/local
DESTDIR ?=
BINDIR ?= $(PREFIX)/bin
DATADIR ?= $(PREFIX)/share
CARGO ?= cargo
TARGET_DIR ?= $(if $(CARGO_TARGET_DIR),$(CARGO_TARGET_DIR),target)
QMAKE ?= $(firstword $(wildcard /usr/lib/qt6/bin/qmake6 /usr/lib64/qt6/bin/qmake6 /usr/lib/qt6/bin/qmake /usr/lib64/qt6/bin/qmake) $(shell command -v qmake6 qmake-qt6 2>/dev/null))
export QMAKE

.PHONY: all build test check install install-data uninstall

all: build

build:
	$(CARGO) build --release --locked

test:
	$(CARGO) test --release --locked

# Everything automated: unit tests, translation catalog, headless scenarios
check: build test
	python3 -I scripts/check-i18n.py
	python3 -I scripts/check-shell-menu.py
	python3 -I scripts/check-ctl-catalog.py
	tests/run.sh
	tests/archive.sh

install: build install-data
	install -Dm755 $(TARGET_DIR)/release/fili-file $(DESTDIR)$(BINDIR)/fili-file
	ln -sf fili-file $(DESTDIR)$(BINDIR)/casier
	ln -sf fili-file $(DESTDIR)$(BINDIR)/fili-archive
	ln -sf fili-file $(DESTDIR)$(BINDIR)/archiveur

install-data:
	install -Dm644 data/fili-file.desktop $(DESTDIR)$(DATADIR)/applications/fili-file.desktop
	install -Dm644 data/fili-archive.desktop $(DESTDIR)$(DATADIR)/applications/fili-archive.desktop
	install -Dm644 data/icons/hicolor/scalable/apps/fili-archive.svg $(DESTDIR)$(DATADIR)/icons/hicolor/scalable/apps/fili-archive.svg
	install -Dm644 man/fili-archive.1 $(DESTDIR)$(DATADIR)/man/man1/fili-archive.1
	ln -sf fili-archive.1 $(DESTDIR)$(DATADIR)/man/man1/archiveur.1
	install -Dm644 data/icons/hicolor/scalable/apps/fili-file.svg $(DESTDIR)$(DATADIR)/icons/hicolor/scalable/apps/fili-file.svg
	install -d $(DESTDIR)$(DATADIR)/dbus-1/services
	sed 's|@BINDIR@|$(BINDIR)|' data/org.freedesktop.impl.portal.desktop.filigrane.service.in > $(DESTDIR)$(DATADIR)/dbus-1/services/org.freedesktop.impl.portal.desktop.filigrane.service
	install -Dm644 data/filigrane.portal $(DESTDIR)$(DATADIR)/xdg-desktop-portal/portals/filigrane.portal
	install -Dm644 fili-file.toml $(DESTDIR)$(DATADIR)/doc/fili-file/fili-file.toml
	install -Dm644 LICENSE $(DESTDIR)$(DATADIR)/licenses/fili-file/LICENSE
	for f in licenses/*; do install -Dm644 "$$f" $(DESTDIR)$(DATADIR)/licenses/fili-file/$$(basename "$$f"); done
	mkdir -p $(DESTDIR)$(DATADIR)/filigrane/shell
	cp -r shell/. $(DESTDIR)$(DATADIR)/filigrane/shell/

uninstall:
	rm -f $(DESTDIR)$(BINDIR)/fili-file $(DESTDIR)$(BINDIR)/casier $(DESTDIR)$(BINDIR)/fili-archive $(DESTDIR)$(BINDIR)/archiveur
	rm -f $(DESTDIR)$(DATADIR)/applications/fili-archive.desktop $(DESTDIR)$(DATADIR)/icons/hicolor/scalable/apps/fili-archive.svg
	rm -f $(DESTDIR)$(DATADIR)/man/man1/fili-archive.1 $(DESTDIR)$(DATADIR)/man/man1/archiveur.1
	rm -f $(DESTDIR)$(DATADIR)/applications/fili-file.desktop
	rm -f $(DESTDIR)$(DATADIR)/doc/fili-file/fili-file.toml
	rm -f $(DESTDIR)$(DATADIR)/icons/hicolor/scalable/apps/fili-file.svg
	rm -f $(DESTDIR)$(DATADIR)/dbus-1/services/org.freedesktop.impl.portal.desktop.filigrane.service
	rm -f $(DESTDIR)$(DATADIR)/xdg-desktop-portal/portals/filigrane.portal
	rm -rf $(DESTDIR)$(DATADIR)/licenses/fili-file
	rm -f $(DESTDIR)$(DATADIR)/filigrane/shell/ctl/fili-file.json $(DESTDIR)$(DATADIR)/filigrane/shell/appmenu/fili-file.json $(DESTDIR)$(DATADIR)/filigrane/shell/rob/fili-file.md
