BUNDLE_PATH ?= "tmp/bundle"
RELEASE ?= "false"
BINARY_PATH ?= "./target/debug/ch-evm"

ifeq ($(RELEASE), true)
	CARGO_BUILD_OPTIONS += "--release"
	BINARY_PATH = "./target/release/ch-evm"
endif

build: 
	cargo clippy
	cargo build ${CARGO_BUILD_OPTIONS}

bundle: build 
	mkdir -p $(BUNDLE_PATH)
	mkdir -p $(BUNDLE_PATH)/etc/clickhouse-server
	mkdir -p $(BUNDLE_PATH)/var/lib/clickhouse/user_defined
	mkdir -p $(BUNDLE_PATH)/var/lib/clickhouse/user_scripts
	mkdir -p $(BUNDLE_PATH)/var/lib/clickhouse/metadata
	cp $(BINARY_PATH) $(BUNDLE_PATH)/var/lib/clickhouse/user_scripts/
	cp config/*_function.*ml $(BUNDLE_PATH)/etc/clickhouse-server/
	cp sql/function_*.sql $(BUNDLE_PATH)/var/lib/clickhouse/user_defined/
	COPYFILE_DISABLE=1 tar --no-xattr -cvzf $(BUNDLE_PATH)/../bundle.tar.gz -C $(BUNDLE_PATH) .

clean:
	rm -rf bin
	rm -rf $(BUNDLE_PATH)/../bundle.tar.gz $(BUNDLE_PATH) $(BUNDLE_PATH)/../bundle.tar.gz
