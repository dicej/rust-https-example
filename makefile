.PHONY: all
all: linux android

.PHONY: linux
linux:
	cargo build --target=x86_64-unknown-linux-gnu

.PHONY: android
android:
	OPENSSL_STATIC=true \
	OPENSSL_DIR=/usr/local/ssl/android-18/ \
	cargo build --target=arm-linux-androideabi
