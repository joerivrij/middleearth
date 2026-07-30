INVENTORY ?= inventories/example/hosts.yml
PLAYBOOK ?= playbooks/site.yml

.PHONY: check erebor lint machines machines-container machines-lima machines-reset syntax

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

erebor:
	@ansible-playbook -i inventories/erebor/hosts.yml playbooks/erebor.yml
