.PHONY: clean build build-mainnet test deploy-devnet deploy-mainnet program-id

# All keypair files are stored OUTSIDE the repo to prevent accidental commits.
# Vault: $(HOME)/.config/flipdash/keys/  (chmod 700)
KEY_VAULT               = $(HOME)/.config/flipdash/keys
DEVNET_PROGRAM_KEYPAIR  = $(KEY_VAULT)/devnet-program.json
MAINNET_PROGRAM_KEYPAIR = $(KEY_VAULT)/mainnet-program-vanity.json
DEPLOY_KEYPAIR          = target/deploy/flipdash_router-keypair.json

DEVNET_UPGRADE_AUTH  = $(KEY_VAULT)/devnet-router-owner.json
MAINNET_UPGRADE_AUTH = $(KEY_VAULT)/mainnet-router-owner-vanity.json

clean:
	@cargo clean

# `cargo clean` removes target/deploy/, including any keypair we placed
# there. The build target picks the right program keypair based on which
# `declare_id!()` is currently set in api/src/lib.rs (devnet vs mainnet).
# Always copy the matching keypair to target/deploy/ before SBF build.
build:
	@mkdir -p target/deploy
	@cp $(DEVNET_PROGRAM_KEYPAIR) $(DEPLOY_KEYPAIR)
	@chmod 600 $(DEPLOY_KEYPAIR)
	@cargo build-sbf
	@echo "Program ID: $$(solana-keygen pubkey $(DEPLOY_KEYPAIR))"
	@echo "Binary:     $$(sha256sum target/deploy/flipdash_router.so)"

build-mainnet:
	@mkdir -p target/deploy
	@cp $(MAINNET_PROGRAM_KEYPAIR) $(DEPLOY_KEYPAIR)
	@chmod 600 $(DEPLOY_KEYPAIR)
	@cargo build-sbf
	@echo "Program ID: $$(solana-keygen pubkey $(DEPLOY_KEYPAIR))"
	@echo "Binary:     $$(sha256sum target/deploy/flipdash_router.so)"

test:
	@cargo build-sbf
	@cargo test-sbf

program-id:
	@solana-keygen pubkey $(DEVNET_PROGRAM_KEYPAIR)
	@solana-keygen pubkey $(MAINNET_PROGRAM_KEYPAIR)

# Devnet deploy. Uses the devnet upgrade-authority keypair from the vault.
# That wallet pays for program rent + tx fees.
deploy-devnet: build
	@solana program deploy \
	  --url devnet \
	  --keypair $(DEVNET_UPGRADE_AUTH) \
	  --program-id $(DEPLOY_KEYPAIR) \
	  --upgrade-authority $(DEVNET_UPGRADE_AUTH) \
	  target/deploy/flipdash_router.so

# Mainnet deploy. Uses the mainnet upgrade-authority keypair from the vault.
# Triple-check the program ID and upgrade authority before running.
deploy-mainnet: build-mainnet
	@solana program deploy \
	  --url mainnet-beta \
	  --keypair $(MAINNET_UPGRADE_AUTH) \
	  --program-id $(DEPLOY_KEYPAIR) \
	  --upgrade-authority $(MAINNET_UPGRADE_AUTH) \
	  target/deploy/flipdash_router.so
