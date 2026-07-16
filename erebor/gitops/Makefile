REPO_URL ?= https://github.com/joerivrij/middleearth.git
BRANCH ?= main
ENV ?= bilbo
PATH_IN_REPO := ./erebor/gitops/cluster/$(ENV)
GITOPS_DIR := $(dir $(abspath $(lastword $(MAKEFILE_LIST))))
EREBOR_DIR := $(abspath $(GITOPS_DIR)/..)
KUBECONFIG ?= $(EREBOR_DIR)/machine/kubeconfig

.PHONY: check-flux check-environment bootstrap-flux bootstrap-bilbo bootstrap-thorin bootstrap-smaug flux-status help

check-flux:
	@command -v flux >/dev/null 2>&1 || { echo "Flux CLI not found" >&2; exit 1; }
	@command -v kubectl >/dev/null 2>&1 || { echo "kubectl not found" >&2; exit 1; }

check-environment:
	@case "$(ENV)" in bilbo|thorin|smaug) ;; *) echo "ENV must be bilbo, thorin, or smaug" >&2; exit 1 ;; esac

bootstrap-flux: check-flux check-environment
	@echo "Bootstrapping Erebor ($(ENV)) from $(PATH_IN_REPO)"
	@KUBECONFIG="$(KUBECONFIG)" flux check --pre
	@KUBECONFIG="$(KUBECONFIG)" flux install
	@KUBECONFIG="$(KUBECONFIG)" flux create source git erebor \
		--url="$(REPO_URL)" \
		--branch="$(BRANCH)" \
		--interval=1m
	@KUBECONFIG="$(KUBECONFIG)" flux create kustomization erebor \
		--source=erebor \
		--path="$(PATH_IN_REPO)" \
		--prune=true \
		--interval=10m

bootstrap-bilbo:
	@$(MAKE) bootstrap-flux ENV=bilbo

bootstrap-thorin:
	@$(MAKE) bootstrap-flux ENV=thorin

bootstrap-smaug:
	@$(MAKE) bootstrap-flux ENV=smaug

flux-status: check-flux
	@KUBECONFIG="$(KUBECONFIG)" flux get all -A

help:
	@echo "make bootstrap-flux [ENV=bilbo|thorin|smaug] [BRANCH=main]"
	@echo "make bootstrap-bilbo | bootstrap-thorin | bootstrap-smaug"
	@echo "make flux-status"
