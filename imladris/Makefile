INVENTORY ?= inventories/example/hosts.yml
PLAYBOOK ?= playbooks/site.yml

.PHONY: check erebor lint machines machines-container machines-lima machines-reset single-node syntax

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

machines-reset:
	@ansible-playbook -i "$(INVENTORY)" playbooks/machines-reset.yml

single-node:
	@ANSIBLE_LOCAL_TEMP=/tmp/imladris-ansible-local ansible-playbook \
		-i inventories/khazad-dum/hosts.yml \
		playbooks/lima-k0s.yml

erebor:
	@ansible-playbook -i inventories/erebor/hosts.yml playbooks/erebor.yml
