# Makefile for tokenization-backend
.DEFAULT_GOAL := help

# ----------------------------------------------------------------------------
# Configuration
# ----------------------------------------------------------------------------
ENV ?= development
ENV_FILE := .env.$(ENV)

# ----------------------------------------------------------------------------
# Docker Compose Commands
# ----------------------------------------------------------------------------
up: ## Start all services in the background with selected env
	@echo "[INFO] Starting stack with $(ENV_FILE)"
	cp $(ENV_FILE) .env
	docker compose up -d

down: ## Stop and remove containers, networks, volumes
	docker compose down

logs: ## Tail logs of all services
	docker compose logs -f

build: ## Build all Docker images
	docker compose build

restart: down up ## Restart the entire stack

ps: ## Show running containers
	docker compose ps

clean: ## Remove containers, networks, volumes, and images
	docker compose down -v --rmi all --remove-orphans

# ----------------------------------------------------------------------------
# Database Migration Commands
# ----------------------------------------------------------------------------
migrate-run: ## Run database migrations
	docker compose run --rm migrate

migrate-revert: ## Revert last database migration (custom wrapper)
	docker compose run --rm migrate sqlx migrate revert

migrate-add: ## Create a new migration with name=$(name)
	docker compose run --rm migrate sqlx migrate add $(name)

# ----------------------------------------------------------------------------
# Smart Contract Deployment (Foundry)
# ----------------------------------------------------------------------------
deploy: ## Deploy upgradeable contracts to target ENV (make deploy ENV=staging)
	@echo "[INFO] Deploying contracts with $(ENV_FILE)"
	cp $(ENV_FILE) .env
	forge script scripts/Deploy.s.sol \
		--rpc-url $$(grep BLOCKCHAIN_RPC_URL .env | cut -d= -f2) \
		--private-key $$(grep BLOCKCHAIN_PRIVATE_KEY .env | cut -d= -f2) \
		--broadcast \
		-vvv

verify: ## Verify contracts on Etherscan/Polygonscan
	@echo "[INFO] Verifying contracts..."
	# example: make verify CONTRACT=TokenFactory ENV=staging
	forge verify-contract \
		--chain-id $$(grep BLOCKCHAIN_CHAIN_ID .env | cut -d= -f2) \
		--compiler-version v0.8.19+commit.7dd6d404 \
		$$(grep CONTRACT_$(CONTRACT) .env | cut -d= -f2) \
		src/contracts/core/$(CONTRACT).sol:$(CONTRACT) \
		$${ETHERSCAN_API_KEY}

# ----------------------------------------------------------------------------
# Utility
# ----------------------------------------------------------------------------
help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'
