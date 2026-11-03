# Casier (fili-file): build and install.
#
#   make                       build the release binary
#   make test                  unit tests
#   sudo make install          install under /usr/local (PREFIX=/usr for a system package)
#   make install DESTDIR=pkg PREFIX=/usr      stage into a directory (packagers)
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

.PHONY: all build test install uninstall

all: build

build:
	$(CARGO) build --release --locked

test:
	$(CARGO) test --release --locked

install: build
	install -Dm755 $(TARGET_DIR)/release/fili-file $(DESTDIR)$(BINDIR)/fili-file
	ln -sf fili-file $(DESTDIR)$(BINDIR)/casier
	install -Dm644 data/fili-file.desktop $(DESTDIR)$(DATADIR)/applications/fili-file.desktop
	install -Dm644 data/icons/hicolor/scalable/apps/fili-file.svg $(DESTDIR)$(DATADIR)/icons/hicolor/scalable/apps/fili-file.svg
	install -d $(DESTDIR)$(DATADIR)/dbus-1/services
	sed 's|@BINDIR@|$(BINDIR)|' data/org.freedesktop.impl.portal.desktop.filigrane.service.in > $(DESTDIR)$(DATADIR)/dbus-1/services/org.freedesktop.impl.portal.desktop.filigrane.service
	install -Dm644 data/filigrane.portal $(DESTDIR)$(DATADIR)/xdg-desktop-portal/portals/filigrane.portal
	install -Dm644 LICENSE $(DESTDIR)$(DATADIR)/licenses/fili-file/LICENSE
	for f in licenses/*; do install -Dm644 "$$f" $(DESTDIR)$(DATADIR)/licenses/fili-file/$$(basename "$$f"); done

uninstall:
	rm -f $(DESTDIR)$(BINDIR)/fili-file $(DESTDIR)$(BINDIR)/casier
	rm -f $(DESTDIR)$(DATADIR)/applications/fili-file.desktop
	rm -f $(DESTDIR)$(DATADIR)/icons/hicolor/scalable/apps/fili-file.svg
	rm -f $(DESTDIR)$(DATADIR)/dbus-1/services/org.freedesktop.impl.portal.desktop.filigrane.service
	rm -f $(DESTDIR)$(DATADIR)/xdg-desktop-portal/portals/filigrane.portal
	rm -rf $(DESTDIR)$(DATADIR)/licenses/fili-file
