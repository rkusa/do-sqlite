.PHONY: dev
dev:
	CC=/opt/homebrew/opt/llvm/bin/clang \
	AR=/opt/homebrew/opt/llvm/bin/llvm-ar \
		wrangler dev

.PHONY: publish
publish:
	CC=/opt/homebrew/opt/llvm/bin/clang \
	AR=/opt/homebrew/opt/llvm/bin/llvm-ar \
		wrangler publish
