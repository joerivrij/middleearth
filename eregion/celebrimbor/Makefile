SSH_HOST ?= bree
SSH_USER ?= pi
SSH_KEY ?= $(HOME)/.ssh/id_raspie
SSH_TARGET := $(SSH_USER)@$(SSH_HOST)
SSH := ssh -o BatchMode=yes -i $(SSH_KEY)
SSH_TTY := ssh -tt -o BatchMode=yes -i $(SSH_KEY)
SCP := scp -o BatchMode=yes -i $(SSH_KEY)

REMOTE_DIR ?= /home/$(SSH_USER)/celebrimbor
REMOTE_BIND ?= 0.0.0.0
PORT ?= 3000
BUILDER_IMAGE ?= celebrimbor-builder
LINUX_TARGET_DIR := $(CURDIR)/target/linux-aarch64
LINUX_BINARY := $(LINUX_TARGET_DIR)/release/celebrimbor
IMAGE_NAME ?= ubuntu-24.04-arm64.qcow2
IMAGE_SOURCE_NAME ?= noble-server-cloudimg-arm64.img
IMAGE_BASE_URL ?= https://cloud-images.ubuntu.com/noble/current
IMAGE_DIR ?= /var/lib/libvirt/images/celebrimbor/src

.PHONY: bootstrap build-linux check-host cloud-init-setup deploy image-bootstrap libvirt-setup libvirt-verify run stop virsh

check-host:
	@$(SSH) $(SSH_TARGET) 'set -eu; \
		uname -m; \
		test -c /dev/kvm; \
		echo "/dev/kvm is available"; \
		command -v virsh; \
		command -v cargo; \
		systemctl is-active --quiet libvirtd; \
		echo "libvirtd is active"'

bootstrap:
	@$(SSH) $(SSH_TARGET) 'set -eu; \
		sudo apt-get update; \
		sudo apt-get install -y \
			build-essential \
			cargo \
			libvirt-clients \
			libvirt-daemon-system \
			libvirt-dev \
			pkg-config \
			qemu-system-arm \
			qemu-utils; \
		sudo usermod -aG libvirt,kvm $(SSH_USER); \
		sudo systemctl enable --now libvirtd'

virsh:
	@$(SSH) $(SSH_TARGET) 'virsh --connect qemu:///system list --all'

libvirt-setup:
	@$(SSH) $(SSH_TARGET) 'set -eu; \
		virsh --connect qemu:///system net-autostart default; \
		if ! virsh --connect qemu:///system net-info default | grep -q "^Active:.*yes"; then \
			virsh --connect qemu:///system net-start default; \
		fi; \
		if ! virsh --connect qemu:///system pool-info celebrimbor >/dev/null 2>&1; then \
			virsh --connect qemu:///system pool-define-as celebrimbor dir \
				--target /var/lib/libvirt/images/celebrimbor; \
			virsh --connect qemu:///system pool-build celebrimbor; \
		fi; \
		virsh --connect qemu:///system pool-autostart celebrimbor; \
		if ! virsh --connect qemu:///system pool-info celebrimbor | grep -q "^State:.*running"; then \
			virsh --connect qemu:///system pool-start celebrimbor; \
		fi'

libvirt-verify:
	@$(SSH) $(SSH_TARGET) 'set -eu; \
		virsh --connect qemu:///system net-list --all; \
		virsh --connect qemu:///system pool-list --all; \
		virsh --connect qemu:///system pool-info celebrimbor'

image-bootstrap:
	@$(SSH) $(SSH_TARGET) 'set -eu; \
		target="$(IMAGE_DIR)/$(IMAGE_NAME)"; \
		if test -f "$$target"; then \
			qemu-img info "$$target"; \
			exit 0; \
		fi; \
		tmp="/tmp/$(IMAGE_SOURCE_NAME).partial"; \
		trap '\''rm -f "$$tmp"'\'' EXIT; \
		curl --fail --location --output "$$tmp" \
			"$(IMAGE_BASE_URL)/$(IMAGE_SOURCE_NAME)"; \
		expected="$$(curl --fail --silent --show-error --location \
			"$(IMAGE_BASE_URL)/SHA256SUMS" | \
			awk '\''$$2 == "*$(IMAGE_SOURCE_NAME)" { print $$1 }'\'')"; \
		test -n "$$expected"; \
		printf "%s  %s\n" "$$expected" "$$tmp" | sha256sum --check -; \
		sudo install -d --mode 0755 "$(IMAGE_DIR)"; \
		sudo install --mode 0644 "$$tmp" "$$target"; \
		virsh --connect qemu:///system pool-refresh celebrimbor; \
		qemu-img info "$$target"'

cloud-init-setup:
	@$(SSH) $(SSH_TARGET) 'set -eu; \
		sudo apt-get update; \
		sudo apt-get install --yes acl cloud-image-utils; \
		sudo setfacl --modify u:$(SSH_USER):rwx "$(dir $(IMAGE_DIR))"; \
		sudo setfacl --modify d:u:$(SSH_USER):rwx "$(dir $(IMAGE_DIR))"'

build-linux:
	@docker build --platform linux/arm64 --tag $(BUILDER_IMAGE) \
		--file Dockerfile.build .
	@docker run --rm --platform linux/arm64 \
		--volume "$(CURDIR):/work" \
		--workdir /work \
		--env CARGO_TARGET_DIR=/work/target/linux-aarch64 \
		$(BUILDER_IMAGE) cargo build --locked --release --features libvirt

deploy: build-linux
	@$(SSH) $(SSH_TARGET) 'mkdir -p $(REMOTE_DIR)'
	@$(SCP) $(LINUX_BINARY) $(SSH_TARGET):$(REMOTE_DIR)/celebrimbor.new
	@$(SCP) $(SSH_KEY).pub $(SSH_TARGET):$(REMOTE_DIR)/authorized_key.pub
	@$(SSH) $(SSH_TARGET) \
		'mv $(REMOTE_DIR)/celebrimbor.new $(REMOTE_DIR)/celebrimbor'

run:
	@$(SSH_TTY) $(SSH_TARGET) 'exec env CELEBRIMBOR_HOST=$(REMOTE_BIND) \
		CELEBRIMBOR_PORT=$(PORT) $(REMOTE_DIR)/celebrimbor'

stop:
	@$(SSH) $(SSH_TARGET) \
		'pkill -f "^$(REMOTE_DIR)/celebrimbor$$" 2>/dev/null || true'
