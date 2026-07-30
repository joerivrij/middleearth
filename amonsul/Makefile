INVENTORY ?= ansible/inventories/example/hosts.yml
ANSIBLE_CONFIG := $(CURDIR)/ansible/ansible.cfg
EXTRA_ARGS ?=

.PHONY: advertise reset setup syntax verify verify-withdrawal withdraw

setup:
	@ANSIBLE_CONFIG="$(ANSIBLE_CONFIG)" ansible-playbook -i "$(INVENTORY)" \
		ansible/playbooks/setup.yml $(EXTRA_ARGS)

verify:
	@ANSIBLE_CONFIG="$(ANSIBLE_CONFIG)" ansible-playbook -i "$(INVENTORY)" \
		ansible/playbooks/verify.yml $(EXTRA_ARGS)

withdraw:
	@ANSIBLE_CONFIG="$(ANSIBLE_CONFIG)" ansible-playbook -i "$(INVENTORY)" \
		ansible/playbooks/withdraw.yml $(EXTRA_ARGS)

verify-withdrawal:
	@ANSIBLE_CONFIG="$(ANSIBLE_CONFIG)" ansible-playbook -i "$(INVENTORY)" \
		ansible/playbooks/verify-withdrawal.yml $(EXTRA_ARGS)

advertise:
	@ANSIBLE_CONFIG="$(ANSIBLE_CONFIG)" ansible-playbook -i "$(INVENTORY)" \
		ansible/playbooks/advertise.yml $(EXTRA_ARGS)

reset:
	@ANSIBLE_CONFIG="$(ANSIBLE_CONFIG)" ansible-playbook -i "$(INVENTORY)" \
		ansible/playbooks/reset.yml $(EXTRA_ARGS)

syntax:
	@for playbook in ansible/playbooks/*.yml; do \
		ANSIBLE_CONFIG="$(ANSIBLE_CONFIG)" ansible-playbook -i "$(INVENTORY)" \
			"$$playbook" --syntax-check || exit 1; \
	done

