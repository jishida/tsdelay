CARGO := $(shell bash --login -c 'command -v cargo')
PREFIX := $(shell if [ -f build/prefix ]; then cat build/prefix; fi)
ifeq ($(PREFIX),)
	PREFIX := /usr/local
endif

all: build

build:
	$(CARGO) build --release
	
check:
	$(CARGO) test

clean:
	$(CARGO) clean
	rm -rf build

install:
	$(CARGO) install --locked --path . --root $(PREFIX)

uninstall:
	$(CARGO) uninstall --root $(PREFIX)

.PHONY: all build check clean install uninstall
