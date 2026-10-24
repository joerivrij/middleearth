INVENTORY ?= inventories/example/hosts.yml
PLAYBOOK ?= playbooks/site.yml
K0S_INVENTORY ?= inventories/khazad-dum/hosts.yml
EREBOR_INVENTORY ?= inventories/erebor/hosts.yml
SMAUG_INVENTORY ?= inventories/smaug/hosts.yml

.PHONY: check erebor lint machines machines-container machines-lima machines-baremetal machines-k0s machines-reset single-node smaug syntax

syntax:
	@for playbook in playbooks/*.yml; do \
		ansible-playbook -i "$(INVENTORY)" "$$playbook" --syntax-check || exit 1; \
	done

lint:
	@ansible-lint

check:
	@ansible-playbook -i "$(INVENTORY)" "$(PLAYBOOK)" --check --diff

machines:
	@ansible-playbook -i "$(INVENTORY)" playbooks/machines.yml

machines-lima:
	@ansible-playbook -i "$(INVENTORY)" playbooks/machines.yml \
		-e machine_backend_filter=lima

machines-container:
	@ansible-playbook -i "$(INVENTORY)" playbooks/machines.yml \
		-e machine_backend_filter=container

machines-baremetal:
	@ansible-playbook -i "$(INVENTORY)" playbooks/machines.yml \
		-e machine_backend_filter=baremetal

machines-k0s:
	@ansible-playbook -i "$(K0S_INVENTORY)" playbooks/machines-k0s.yml

machines-reset:
	@ansible-playbook -i "$(INVENTORY)" playbooks/machines-reset.yml

single-node: machines-k0s

erebor:
	@ansible-playbook -i "$(EREBOR_INVENTORY)" playbooks/erebor.yml

smaug:
	@ansible-playbook -i "$(SMAUG_INVENTORY)" playbooks/smaug.yml
