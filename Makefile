# One command installs, one runs, one tests (the run scripts do the same, and more).
#   make install   build omnicui, the apps and their targets (needs Go, Node, Java 21, Rust)
#   make check     run every contract on every target
#   make test      everything: unit tests, failure injection, the real targets, the report's accessibility
#   make release VERSION=1.2.3   omnicui for every platform, versioned, and its source, with checksums, in dist/

GO      ?= go
NPM     ?= npm
CARGO   ?= cargo
OMNICUI := $(CURDIR)/bin/omnicui
OMNI    := $(OMNICUI) --project $(CURDIR)/omnicui
VERSION ?= $(shell cat VERSION 2>/dev/null || echo dev)
PLATFORMS := linux/amd64 linux/arm64 darwin/amd64 darwin/arm64 windows/amd64 windows/arm64

.PHONY: install omnicui web javafx egui report-tests check test test-unit test-real test-report cross release clean

install: omnicui web javafx egui report-tests

omnicui:
	cd tools/omnicui && $(GO) build -trimpath -o $(OMNICUI) ./cmd/omnicui

# Each app first, then the omnicui target that imports it.
web:
	cd apps/web && $(NPM) ci --no-audit --no-fund
	cd omnicui/targets/web && $(NPM) ci --no-audit --no-fund && npx playwright install chromium
	$(OMNI) stubs web
	cd apps/web && npx tsc --noEmit
	cd omnicui/targets/web && npx tsc --noEmit

javafx:
	cd apps/desktop-javafx && ./mvnw -q -B install
	cd omnicui/targets/javafx && ./mvnw -q -B package

egui:
	$(OMNI) stubs egui
	cd apps/desktop && $(CARGO) build --release --locked
	cd omnicui/targets/egui && $(CARGO) build --release --locked

report-tests:
	cd tools/omnicui/report-test && $(NPM) ci --no-audit --no-fund

check:
	$(OMNI) check

test: test-unit test-real test-report cross

# omnicui's own tests: contracts, protocol, failure injection against a fake adapter, the CLI.
test-unit:
	cd tools/omnicui && $(GO) vet ./... && $(GO) test -count=1 ./...

# The real targets, the deliberate defects (each one must be caught), the stubs.
test-real:
	cd tools/omnicui && OMNICUI_BIN=$(OMNICUI) $(GO) test -count=1 -tags real -timeout 40m -v ./integration/

# report.html in a real browser: axe (WCAG 2.2 AA) and its layout at desktop, phone, dark and forced colours.
test-report:
	OMNICUI_DEFECTS=javafx.adapter.stdout,egui.counter.small-text,javafx.switch.no-label,web.switch.slow $(OMNI) check > /dev/null; cd tools/omnicui/report-test && REPORT_LABEL=failing node report.test.ts
	$(OMNI) check > /dev/null; cd tools/omnicui/report-test && REPORT_LABEL=passing node report.test.ts

# omnicui builds for every platform it supports.
cross:
	cd tools/omnicui && for p in $(PLATFORMS); do \
	  GOOS=$${p%/*} GOARCH=$${p#*/} $(GO) build -trimpath -o /dev/null ./cmd/omnicui && echo "built for $$p"; done

# What ships: one static binary per platform, stamped with VERSION; the command's source
# (tools/omnicui without the real-target and report tests, which need this repository's apps), for
# projects that vendor and build it; and SHA256SUMS.
release:
	rm -rf dist && mkdir -p dist
	cd tools/omnicui && for p in $(PLATFORMS); do \
	  os=$${p%/*}; arch=$${p#*/}; ext=; [ $$os = windows ] && ext=.exe; \
	  CGO_ENABLED=0 GOOS=$$os GOARCH=$$arch $(GO) build -trimpath -ldflags "-s -w -X main.Version=$(VERSION)" \
	    -o $(CURDIR)/dist/omnicui_$(VERSION)_$${os}_$${arch}$$ext ./cmd/omnicui || exit 1; done
	tar -czf dist/omnicui-src-$(VERSION).tar.gz --exclude=integration --exclude=report-test -C tools omnicui
	cd dist && (command -v sha256sum >/dev/null && sha256sum omnicui_* omnicui-src-* || shasum -a 256 omnicui_* omnicui-src-*) > SHA256SUMS
	@cat dist/SHA256SUMS

clean:
	rm -rf bin dist omnicui/.omnicui apps/web/node_modules apps/web/dist apps/desktop/target apps/desktop-javafx/target \
	  omnicui/targets/web/node_modules omnicui/targets/web/dist omnicui/targets/javafx/target omnicui/targets/egui/target \
	  tools/omnicui/report-test/node_modules
